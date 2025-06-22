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
    #[error("Unable to load Segmentation Model")]
    SegmentModelLoadError,

    #[error("Unable to load segments {0}")]
    SegmentInferenceError(#[from] kalosm::vision::SegmentAnythingInferenceError),
}