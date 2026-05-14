use crate::error::ConvertError;

const SECTOR: usize = 2048;
const PVD_SECTOR: usize = 16;

pub struct IsoMeta {
    pub title: String,
    pub title_id: String,
}

pub fn extract_iso_meta(iso: &[u8]) -> Result<IsoMeta, ConvertError> {
    let pvd_start = PVD_SECTOR * SECTOR;
    let pvd = iso
        .get(pvd_start..pvd_start + SECTOR)
        .ok_or_else(|| ConvertError::InvalidFile("ISO too small for PVD".into()))?;

    if pvd[0] != 1 || &pvd[1..6] != b"CD001" {
        return Err(ConvertError::InvalidFile("Not a valid ISO9660 image".into()));
    }

    let volume_id = &pvd[40..72];
    let title = String::from_utf8_lossy(volume_id)
        .trim_end()
        .to_string();

    let title_id = extract_title_id_from_iso(iso).unwrap_or_else(|| "ULUS00000".to_string());

    Ok(IsoMeta {
        title: if title.is_empty() { "Unknown".to_string() } else { title },
        title_id,
    })
}

fn extract_title_id_from_iso(iso: &[u8]) -> Option<String> {
    let pvd_start = PVD_SECTOR * SECTOR;
    let pvd = iso.get(pvd_start..pvd_start + SECTOR)?;

    let root_dir_start = 156usize;
    let root_dir_entry = pvd.get(root_dir_start..root_dir_start + 34)?;

    let lba = u32::from_le_bytes([
        root_dir_entry[2],
        root_dir_entry[3],
        root_dir_entry[4],
        root_dir_entry[5],
    ]) as usize;
    let dir_size = u32::from_le_bytes([
        root_dir_entry[10],
        root_dir_entry[11],
        root_dir_entry[12],
        root_dir_entry[13],
    ]) as usize;

    let dir_start = lba * SECTOR;
    let dir_data = iso.get(dir_start..dir_start + dir_size)?;

    let mut pos = 0;
    while pos < dir_data.len() {
        let rec_len = dir_data[pos] as usize;
        if rec_len == 0 {
            pos = (pos / SECTOR + 1) * SECTOR;
            if pos >= dir_data.len() {
                break;
            }
            continue;
        }

        let name_len = dir_data.get(pos + 32).copied()? as usize;
        let name_bytes = dir_data.get(pos + 33..pos + 33 + name_len)?;
        let name = String::from_utf8_lossy(name_bytes);
        let name = name.trim_end_matches(";1");

        if name == "PARAM.SFO" || name == "UMD_DATA.BIN" {
            let file_lba = u32::from_le_bytes([
                dir_data[pos + 2],
                dir_data[pos + 3],
                dir_data[pos + 4],
                dir_data[pos + 5],
            ]) as usize;
            let file_size = u32::from_le_bytes([
                dir_data[pos + 10],
                dir_data[pos + 11],
                dir_data[pos + 12],
                dir_data[pos + 13],
            ]) as usize;
            let file_start = file_lba * SECTOR;
            let file_data = iso.get(file_start..file_start + file_size)?;

            if name == "PARAM.SFO" {
                if let Ok(sfo) = crate::converter::sfo::Sfo::parse(file_data) {
                    if let Some(id) = sfo.get_str("DISC_ID") {
                        return Some(id.to_string());
                    }
                }
            } else {
                let s = std::str::from_utf8(file_data).ok()?;
                let id = s.split('|').next()?.trim().to_string();
                if id.len() == 9 {
                    return Some(id);
                }
            }
        }

        pos += rec_len;
    }

    None
}
