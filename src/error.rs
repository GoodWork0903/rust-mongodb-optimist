use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("MongoDb error: {0}")]
    MongoDb(#[from] mongodb::error::Error),
    #[error("SerdeYaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Tokio error: {0}")]
    Tokio(#[from] tokio::task::JoinError),
    #[error("ZipArchive Error: {0}")]
    Zip(#[from] zip::result::ZipError),
}
