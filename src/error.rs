use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiftError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Engine export error: {0}")]
    Engine(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("Glob error: {0}")]
    Glob(#[from] glob::PatternError),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RiftError>;
