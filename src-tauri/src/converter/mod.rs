pub mod decompress;
pub mod iso;
pub mod pbp;
pub mod sfo;
pub mod vpk;

use std::path::Path;

use crate::error::ConvertError;
use decompress::decompress_cso;
use iso::extract_iso_meta;
use pbp::parse_pbp;
use sfo::Sfo;
use vpk::{VpkMeta, build_vpk};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConvertOptions {
    pub input_path: String,
    pub output_dir: String,
    pub title_override: Option<String>,
    pub title_id_override: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConvertResult {
    pub output_path: String,
    pub title: String,
    pub title_id: String,
    pub format_detected: String,
}

pub fn convert(opts: &ConvertOptions) -> Result<ConvertResult, ConvertError> {
    let input = Path::new(&opts.input_path);
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    let (iso_data, meta_sfo_bytes, icon0, pic1, format_detected) = match ext.as_str() {
        "iso" => {
            let data = std::fs::read(input)?;
            (data, None::<Vec<u8>>, None, None, "ISO".to_string())
        }
        "cso" | "zso" => {
            let raw = std::fs::read(input)?;
            let data = decompress_cso(&raw)?;
            (data, None, None, None, ext.to_uppercase())
        }
        "pbp" => {
            let raw = std::fs::read(input)?;
            let pbp = parse_pbp(&raw)?;
            (
                pbp.data_psar,
                Some(pbp.param_sfo),
                pbp.icon0,
                pbp.pic1,
                "PBP".to_string(),
            )
        }
        other => {
            return Err(ConvertError::UnsupportedFormat(format!(
                ".{} is not supported. Use ISO, CSO, ZSO, or PBP.",
                other
            )));
        }
    };

    let (title, title_id, version, psp_sfo_bytes) = if let Some(sfo_bytes) = meta_sfo_bytes {
        let sfo = Sfo::parse(&sfo_bytes)?;
        let title = opts
            .title_override
            .clone()
            .unwrap_or_else(|| sfo.get_str("TITLE").unwrap_or("Unknown").to_string());
        let title_id = opts
            .title_id_override
            .clone()
            .unwrap_or_else(|| sfo.get_str("DISC_ID").unwrap_or("ULUS00000").to_string());
        let version = sfo.get_str("DISC_VERSION").unwrap_or("01.00").to_string();
        (title, title_id, version, sfo_bytes)
    } else {
        let iso_meta = extract_iso_meta(&iso_data)?;
        let title = opts
            .title_override
            .clone()
            .unwrap_or(iso_meta.title.clone());
        let title_id = opts
            .title_id_override
            .clone()
            .unwrap_or(iso_meta.title_id.clone());
        let sfo = build_psp_sfo(&title, &title_id, "01.00");
        (title, title_id, "01.00".to_string(), sfo)
    };

    let sanitized_title: String = title
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let filename = format!("{}-{}.vpk", sanitized_title.trim().replace(' ', "_"), title_id);
    let output_path = Path::new(&opts.output_dir).join(&filename);

    let vpk_meta = VpkMeta {
        title: title.clone(),
        title_id: title_id.clone(),
        version,
        icon0,
        pic1,
        iso_data,
        param_sfo_psp: psp_sfo_bytes,
    };

    let vpk_bytes = build_vpk(&vpk_meta)?;
    std::fs::write(&output_path, &vpk_bytes)?;

    Ok(ConvertResult {
        output_path: output_path.to_string_lossy().to_string(),
        title,
        title_id,
        format_detected,
    })
}

fn build_psp_sfo(title: &str, title_id: &str, version: &str) -> Vec<u8> {
    use sfo::SfoBuilder;
    SfoBuilder::new()
        .utf8("CATEGORY", "UG", 4)
        .utf8("DISC_ID", title_id, 12)
        .utf8("DISC_VERSION", version, 8)
        .utf8("TITLE", title, 128)
        .u32("BOOTABLE", 1)
        .u32("REGION", 0x8000)
        .build()
}
