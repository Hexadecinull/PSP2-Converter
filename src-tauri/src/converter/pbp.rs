use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use crate::error::ConvertError;

const PBP_MAGIC: u32 = 0x50425000;

pub struct PbpFile {
    pub param_sfo: Vec<u8>,
    pub icon0: Option<Vec<u8>>,
    pub pic1: Option<Vec<u8>>,
    pub data_psar: Vec<u8>,
}

pub fn parse_pbp(data: &[u8]) -> Result<PbpFile, ConvertError> {
    let mut c = Cursor::new(data);

    let magic = c
        .read_u32::<LittleEndian>()
        .map_err(|_| ConvertError::PbpError("truncated magic".into()))?;
    if magic != PBP_MAGIC {
        return Err(ConvertError::PbpError("bad PBP magic".into()));
    }

    let _version = c.read_u32::<LittleEndian>().ok();

    let mut offsets = [0u32; 8];
    for o in offsets.iter_mut() {
        *o = c
            .read_u32::<LittleEndian>()
            .map_err(|_| ConvertError::PbpError("truncated offsets".into()))?;
    }

    let read_section = |start: u32, end: u32| -> Option<Vec<u8>> {
        let s = start as usize;
        let e = end as usize;
        if s >= e || e > data.len() {
            return None;
        }
        Some(data[s..e].to_vec())
    };

    let param_sfo = read_section(offsets[0], offsets[1])
        .ok_or_else(|| ConvertError::PbpError("missing PARAM.SFO section".into()))?;
    let icon0 = read_section(offsets[1], offsets[2]);
    let pic1 = read_section(offsets[4], offsets[5]);
    let data_psar = read_section(offsets[6], offsets[7].max(data.len() as u32))
        .unwrap_or_else(|| {
            let s = offsets[6] as usize;
            if s < data.len() {
                data[s..].to_vec()
            } else {
                Vec::new()
            }
        });

    Ok(PbpFile {
        param_sfo,
        icon0,
        pic1,
        data_psar,
    })
}
