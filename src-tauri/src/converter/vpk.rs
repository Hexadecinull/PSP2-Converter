use std::io::{Cursor, Write};
use zip::{write::FileOptions, ZipWriter};

use crate::error::ConvertError;

pub struct VpkMeta {
    pub title: String,
    pub title_id: String,
    pub version: String,
    pub icon0: Option<Vec<u8>>,
    pub pic1: Option<Vec<u8>>,
    pub iso_data: Vec<u8>,
    pub param_sfo_psp: Vec<u8>,
}

static PLACEHOLDER_PNG_1x1: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
    0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41,
    0x54, 0x08, 0xd7, 0x63, 0x18, 0x18, 0x18, 0x00,
    0x00, 0x00, 0x04, 0x00, 0x01, 0xa3, 0x17, 0xdb,
    0x6b, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
    0x44, 0xae, 0x42, 0x60, 0x82,
];

pub fn build_vpk(meta: &VpkMeta) -> Result<Vec<u8>, ConvertError> {
    let vita_sfo = build_vita_sfo(meta);
    let eboot_pbp = build_eboot_pbp(meta)?;
    let livearea_bg = build_livearea_bg();

    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);

    let stored: FileOptions = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    let deflated: FileOptions = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("sce_sys/param.sfo", stored.clone())
        .map_err(ConvertError::ZipError)?;
    zip.write_all(&vita_sfo)
        .map_err(ConvertError::Io)?;

    let icon = meta.icon0.as_deref().unwrap_or(PLACEHOLDER_PNG_1x1);
    zip.start_file("sce_sys/icon0.png", stored.clone())
        .map_err(ConvertError::ZipError)?;
    zip.write_all(icon)
        .map_err(ConvertError::Io)?;

    zip.start_file("sce_sys/livearea/contents/bg.png", stored.clone())
        .map_err(ConvertError::ZipError)?;
    zip.write_all(meta.pic1.as_deref().unwrap_or(&livearea_bg))
        .map_err(ConvertError::Io)?;

    zip.start_file("sce_sys/livearea/contents/template.xml", stored.clone())
        .map_err(ConvertError::ZipError)?;
    zip.write_all(livearea_template(&meta.title).as_bytes())
        .map_err(ConvertError::Io)?;

    zip.start_file("eboot.bin", stored.clone())
        .map_err(ConvertError::ZipError)?;
    zip.write_all(&eboot_pbp)
        .map_err(ConvertError::Io)?;

    let result = zip.finish().map_err(ConvertError::ZipError)?;
    Ok(result.into_inner())
}

fn build_vita_sfo(meta: &VpkMeta) -> Vec<u8> {
    use crate::converter::sfo::SfoBuilder;

    SfoBuilder::new()
        .utf8("CATEGORY", "ME", 4)
        .utf8("STITLE", &meta.title, 52)
        .utf8("TITLE", &meta.title, 128)
        .utf8("TITLE_ID", &meta.title_id, 12)
        .utf8("VERSION", &meta.version, 8)
        .u32("ATTRIBUTE2", 0)
        .u32("BOOT_FILE", 0)
        .u32("CONTENT_ID", 0)
        .build()
}

fn build_eboot_pbp(meta: &VpkMeta) -> Result<Vec<u8>, ConvertError> {
    use byteorder::{LittleEndian, WriteBytesExt};

    let sections: [&[u8]; 8] = [
        &meta.param_sfo_psp,
        meta.icon0.as_deref().unwrap_or(PLACEHOLDER_PNG_1x1),
        PLACEHOLDER_PNG_1x1,
        PLACEHOLDER_PNG_1x1,
        meta.pic1.as_deref().unwrap_or(PLACEHOLDER_PNG_1x1),
        &[],
        &meta.iso_data,
        &[],
    ];

    let header_size: u32 = 4 + 4 + 8 * 4;
    let mut offsets = [0u32; 8];
    let mut running: u32 = header_size;
    for (i, sec) in sections.iter().enumerate() {
        offsets[i] = running;
        running += sec.len() as u32;
    }

    let mut out: Vec<u8> = Vec::new();
    out.write_u32::<LittleEndian>(0x50425000).map_err(ConvertError::Io)?;
    out.write_u32::<LittleEndian>(0x00010000).map_err(ConvertError::Io)?;
    for &o in &offsets {
        out.write_u32::<LittleEndian>(o).map_err(ConvertError::Io)?;
    }
    for sec in &sections {
        out.extend_from_slice(sec);
    }

    Ok(out)
}

fn build_livearea_bg() -> Vec<u8> {
    PLACEHOLDER_PNG_1x1.to_vec()
}

fn livearea_template(title: &str) -> String {
    let escaped = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<livearea style="psmobile-game-display" template-version="1.0" content-rev="1">
  <livearea-background>
    <image>bg.png</image>
  </livearea-background>
  <gate>
    <gate-icon-transition style="dissolve"/>
    <startup-background>
      <color>#000000</color>
    </startup-background>
    <startup-image>
      <image>bg.png</image>
    </startup-image>
    <label>{}</label>
  </gate>
</livearea>"#,
        escaped
    )
}
