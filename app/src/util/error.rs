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

    #[error("Ollama error: {0}")]
    OllamaError(String),    

    #[error("Kalosm error: {0}")]
    KalosmError(String),

    #[error("Rten error: {0}")]
    RtenError(String),

    #[error("Ocrs error: {0}")]
    OcrsError(String),

    #[error("ImageSource error: {0}")]
    ImageSourceError(String),

    #[error("PaddleOcr error: {0}")]
    PaddleOcrError(String),
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

impl From<ollama_rs::error::OllamaError> for ModelError {
    fn from(err: ollama_rs::error::OllamaError) -> Self {
        ModelError::OllamaError(err.to_string())
    }
}

// impl From<kalosm::vision::LoadOcrError> for ModelError {
//     fn from(err: kalosm::vision::LoadOcrError) -> Self {
//         ModelError::KalosmError(err.to_string())
//     }
// }

// impl From<kalosm::vision::OcrInferenceError> for ModelError {
//     fn from(err: kalosm::vision::OcrInferenceError) -> Self {
//         ModelError::KalosmError(err.to_string())
//     }
// }

impl From<rten::ModelLoadError> for ModelError {
    fn from(err: rten::ModelLoadError) -> Self {
        ModelError::RtenError(err.to_string())
    }
}

impl From<anyhow::Error> for ModelError {
    fn from(err: anyhow::Error) -> Self {
        ModelError::OcrsError(err.to_string())
    }
}

impl From<ocrs::ImageSourceError> for ModelError {
    fn from(err: ocrs::ImageSourceError) -> Self {
        ModelError::ImageSourceError(err.to_string())
    }
}

impl From<paddle_ocr_rs::ocr_error::OcrError> for ModelError {
    fn from(err: paddle_ocr_rs::ocr_error::OcrError) -> Self {
        ModelError::PaddleOcrError(err.to_string())
    }
}