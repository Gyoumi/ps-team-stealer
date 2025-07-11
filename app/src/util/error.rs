use thiserror::Error;

#[derive(Error, Debug)]
pub enum YtDlpError {
    #[error("Failed to execute yt-dlp: {0}")]
    CommandError(String),

    #[error("Failed to parse yt-dlp output: {0}")]
    ParseError(String),

    #[error("Required field missing: {0}")]
    MissingField(String),

    #[error("No suitable video-only format found")]
    NoVideoOnlyFormat,
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Image error: {0}")]
    ImageError(String),

    #[error("ONNX error: {0}")]
    OnnxError(String),
}

impl From<image::ImageError> for ModelError {
    fn from(err: image::ImageError) -> Self {
        ModelError::ImageError(err.to_string())
    }
}

impl From<ort::Error> for ModelError {
    fn from(err: ort::Error) -> Self {
        ModelError::OnnxError(err.to_string())
    }
}