use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::error::ConvertError;

const CSO_MAGIC: u32 = 0x4F534943;
const ZSO_MAGIC: u32 = 0x4F53495A;

pub fn decompress_cso(data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let mut c = Cursor::new(data);

    let magic = c
        .read_u32::<LittleEndian>()
        .map_err(|_| ConvertError::CsoError("truncated header".into()))?;
    let is_zso = match magic {
        CSO_MAGIC => false,
        ZSO_MAGIC => true,
        _ => return Err(ConvertError::CsoError("bad magic".into())),
    };

    let _header_size = c
        .read_u32::<LittleEndian>()
        .map_err(|_| ConvertError::CsoError("truncated header size".into()))?;
    let total_bytes = c
        .read_u64::<LittleEndian>()
        .map_err(|_| ConvertError::CsoError("truncated total bytes".into()))?;
    let block_size = c
        .read_u32::<LittleEndian>()
        .map_err(|_| ConvertError::CsoError("truncated block size".into()))?;
    let version = c
        .read_u8()
        .map_err(|_| ConvertError::CsoError("truncated version".into()))?;
    let _index_shift = c
        .read_u8()
        .map_err(|_| ConvertError::CsoError("truncated index shift".into()))?;
    c.seek(SeekFrom::Current(2)).ok();

    if version != 1 {
        return Err(ConvertError::CsoError(format!(
            "unsupported CSO/ZSO version {}",
            version
        )));
    }

    let num_blocks = (total_bytes + block_size as u64 - 1) / block_size as u64;
    let index_count = (num_blocks + 1) as usize;

    let mut index: Vec<u32> = Vec::with_capacity(index_count);
    for _ in 0..index_count {
        index.push(
            c.read_u32::<LittleEndian>()
                .map_err(|_| ConvertError::CsoError("truncated block index".into()))?,
        );
    }

    let mut out: Vec<u8> = Vec::with_capacity(total_bytes as usize);

    for block in 0..num_blocks as usize {
        let raw_offset = index[block];
        let next_offset = index[block + 1];
        let compressed = (raw_offset & 0x8000_0000) == 0;
        let offset = (raw_offset & 0x7FFF_FFFF) as u64;
        let next = (next_offset & 0x7FFF_FFFF) as u64;
        let size = (next - offset) as usize;

        c.seek(SeekFrom::Start(offset))
            .map_err(|_| ConvertError::CsoError("seek error".into()))?;

        let mut block_data = vec![0u8; size];
        c.read_exact(&mut block_data)
            .map_err(|_| ConvertError::CsoError("block read error".into()))?;

        if !compressed {
            out.write_all(&block_data)
                .map_err(|_| ConvertError::CsoError("write error".into()))?;
        } else if is_zso {
            let decompressed = lz4_flex::decompress(&block_data, block_size as usize)
                .map_err(|e| ConvertError::ZsoError(e.to_string()))?;
            out.write_all(&decompressed)
                .map_err(|_| ConvertError::ZsoError("write error".into()))?;
        } else {
            use flate2::read::DeflateDecoder;
            let mut decoder = DeflateDecoder::new(block_data.as_slice());
            let mut dec = Vec::new();
            decoder
                .read_to_end(&mut dec)
                .map_err(|e| ConvertError::CsoError(e.to_string()))?;
            out.write_all(&dec)
                .map_err(|_| ConvertError::CsoError("write error".into()))?;
        }
    }

    out.truncate(total_bytes as usize);
    Ok(out)
}
