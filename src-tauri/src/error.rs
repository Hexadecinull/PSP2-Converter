use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid file: {0}")]
    InvalidFile(String),

    #[error("CSO decompression failed: {0}")]
    CsoError(String),

    #[error("ZSO decompression failed: {0}")]
    ZsoError(String),

    #[error("PBP parse error: {0}")]
    PbpError(String),

    #[error("SFO parse error: {0}")]
    SfoError(String),

    #[error("VPK build error: {0}")]
    VpkError(String),

    #[error("Zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),
}

impl serde::Serialize for ConvertError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
