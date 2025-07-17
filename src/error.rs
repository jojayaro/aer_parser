use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Date parsing error: {0}")]
    ChronoParse(#[from] chrono::ParseError),
    #[error("File processing error: {0}")]
    FileProcessing(String),
    #[error("Download error: {0}")]
    Download(String),
    #[error("Invalid command line arguments: {0}")]
    Cli(String),
}
