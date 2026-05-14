use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use crate::error::ConvertError;

const SFO_MAGIC: u32 = 0x46535000;

#[derive(Debug, Clone)]
pub enum SfoValue {
    Utf8(String),
    Utf8Special(String),
    U32(u32),
}

#[derive(Debug, Clone)]
pub struct Sfo {
    pub entries: HashMap<String, SfoValue>,
}

impl Sfo {
    pub fn parse(data: &[u8]) -> Result<Self, ConvertError> {
        let mut cursor = Cursor::new(data);

        let magic = cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| ConvertError::SfoError("truncated header".into()))?;
        if magic != SFO_MAGIC {
            return Err(ConvertError::SfoError("bad magic".into()));
        }

        let _version = cursor.read_u32::<LittleEndian>().ok();
        let key_table_offset = cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| ConvertError::SfoError("truncated key table offset".into()))?;
        let data_table_offset = cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| ConvertError::SfoError("truncated data table offset".into()))?;
        let num_entries = cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| ConvertError::SfoError("truncated entry count".into()))?;

        let mut raw: Vec<(u16, u8, u32, u32, u32)> = Vec::new();

        for _ in 0..num_entries {
            let key_offset = cursor
                .read_u16::<LittleEndian>()
                .map_err(|_| ConvertError::SfoError("truncated index".into()))?;
            let fmt = cursor
                .read_u8()
                .map_err(|_| ConvertError::SfoError("truncated fmt".into()))?;
            let _align = cursor
                .read_u8()
                .map_err(|_| ConvertError::SfoError("truncated align".into()))?;
            let data_len = cursor
                .read_u32::<LittleEndian>()
                .map_err(|_| ConvertError::SfoError("truncated data len".into()))?;
            let data_max_len = cursor
                .read_u32::<LittleEndian>()
                .map_err(|_| ConvertError::SfoError("truncated data max len".into()))?;
            let data_offset = cursor
                .read_u32::<LittleEndian>()
                .map_err(|_| ConvertError::SfoError("truncated data offset".into()))?;
            raw.push((key_offset, fmt, data_len, data_max_len, data_offset));
        }

        let mut entries = HashMap::new();

        for (key_offset, fmt, data_len, _data_max_len, data_offset) in raw {
            let key_pos = (key_table_offset + key_offset as u32) as usize;
            let key = read_cstr(data, key_pos).map_err(|e| ConvertError::SfoError(e))?;

            let val_pos = (data_table_offset + data_offset) as usize;

            let value = match fmt {
                0x04 => {
                    let s = data
                        .get(val_pos..val_pos + data_len as usize)
                        .ok_or_else(|| ConvertError::SfoError("data slice oob".into()))?;
                    let s = std::str::from_utf8(s)
                        .map_err(|_| ConvertError::SfoError("invalid utf8".into()))?
                        .trim_end_matches('\0')
                        .to_string();
                    SfoValue::Utf8Special(s)
                }
                0x204 => {
                    let s = data
                        .get(val_pos..val_pos + data_len as usize)
                        .ok_or_else(|| ConvertError::SfoError("data slice oob".into()))?;
                    let s = std::str::from_utf8(s)
                        .map_err(|_| ConvertError::SfoError("invalid utf8".into()))?
                        .trim_end_matches('\0')
                        .to_string();
                    SfoValue::Utf8(s)
                }
                0x404 => {
                    let bytes = data
                        .get(val_pos..val_pos + 4)
                        .ok_or_else(|| ConvertError::SfoError("u32 slice oob".into()))?;
                    let v = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    SfoValue::U32(v)
                }
                _ => continue,
            };

            entries.insert(key, value);
        }

        Ok(Sfo { entries })
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(SfoValue::Utf8(s)) | Some(SfoValue::Utf8Special(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_u32(&self, key: &str) -> Option<u32> {
        match self.entries.get(key) {
            Some(SfoValue::U32(v)) => Some(*v),
            _ => None,
        }
    }
}

fn read_cstr(data: &[u8], start: usize) -> Result<String, String> {
    let slice = data.get(start..).ok_or("key offset oob")?;
    let end = slice.iter().position(|&b| b == 0).ok_or("unterminated key string")?;
    std::str::from_utf8(&slice[..end])
        .map(|s| s.to_string())
        .map_err(|_| "key not utf8".into())
}

pub struct SfoBuilder {
    entries: Vec<SfoEntry>,
}

struct SfoEntry {
    key: String,
    value: SfoValue,
    max_len: u32,
}

impl SfoBuilder {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn utf8(mut self, key: &str, value: &str, max_len: u32) -> Self {
        self.entries.push(SfoEntry {
            key: key.to_string(),
            value: SfoValue::Utf8(value.to_string()),
            max_len,
        });
        self
    }

    pub fn u32(mut self, key: &str, value: u32) -> Self {
        self.entries.push(SfoEntry {
            key: key.to_string(),
            value: SfoValue::U32(value),
            max_len: 4,
        });
        self
    }

    pub fn build(mut self) -> Vec<u8> {
        self.entries.sort_by(|a, b| a.key.cmp(&b.key));

        let n = self.entries.len() as u32;
        let index_size = n * 16;
        let header_size = 20u32;
        let index_offset = header_size;

        let mut key_table: Vec<u8> = Vec::new();
        let mut key_offsets: Vec<u16> = Vec::new();
        for e in &self.entries {
            key_offsets.push(key_table.len() as u16);
            key_table.extend_from_slice(e.key.as_bytes());
            key_table.push(0);
        }
        while key_table.len() % 4 != 0 {
            key_table.push(0);
        }

        let key_table_offset = index_offset + index_size;
        let data_table_offset = key_table_offset + key_table.len() as u32;

        let mut data_table: Vec<u8> = Vec::new();
        let mut data_offsets: Vec<u32> = Vec::new();
        let mut data_lens: Vec<u32> = Vec::new();

        for e in &self.entries {
            data_offsets.push(data_table.len() as u32);
            match &e.value {
                SfoValue::Utf8(s) | SfoValue::Utf8Special(s) => {
                    let bytes = s.as_bytes();
                    let len = bytes.len() + 1;
                    data_lens.push(len as u32);
                    data_table.extend_from_slice(bytes);
                    data_table.push(0);
                    let padded = (e.max_len as usize).max(len);
                    while data_table.len() < (data_offsets.last().copied().unwrap_or(0) as usize + padded) {
                        data_table.push(0);
                    }
                }
                SfoValue::U32(v) => {
                    data_lens.push(4);
                    data_table.write_u32::<LittleEndian>(*v).unwrap();
                }
            }
        }

        let mut out = Vec::new();
        out.write_u32::<LittleEndian>(SFO_MAGIC).unwrap();
        out.write_u32::<LittleEndian>(0x0101).unwrap();
        out.write_u32::<LittleEndian>(key_table_offset).unwrap();
        out.write_u32::<LittleEndian>(data_table_offset).unwrap();
        out.write_u32::<LittleEndian>(n).unwrap();

        for (i, e) in self.entries.iter().enumerate() {
            let fmt: u16 = match &e.value {
                SfoValue::Utf8(_) => 0x0204,
                SfoValue::Utf8Special(_) => 0x0004,
                SfoValue::U32(_) => 0x0404,
            };
            out.write_u16::<LittleEndian>(key_offsets[i]).unwrap();
            out.write_u16::<LittleEndian>(fmt).unwrap();
            out.write_u32::<LittleEndian>(data_lens[i]).unwrap();
            out.write_u32::<LittleEndian>(e.max_len).unwrap();
            out.write_u32::<LittleEndian>(data_offsets[i]).unwrap();
        }

        out.extend_from_slice(&key_table);
        out.extend_from_slice(&data_table);
        out
    }
}
