//! Trait definitions for report parsing

use chrono::NaiveDate;
use serde::Serialize;

use crate::AppError;

/// Common trait for all AER report parsers
pub trait ReportParser {
    /// The output type for this parser
    type Output: Serialize;
    
    /// Parse a single report file
    fn parse_file(&self, content: &str) -> Result<Vec<Self::Output>, ParseError>;
    
    /// Extract the report date
    fn extract_date(&self, content: &str) -> Result<NaiveDate, ParseError>;
    
    /// Get the CSV filename prefix for this report type
    fn csv_prefix(&self) -> &'static str;
}

/// Parsing context for shared operations
#[derive(Debug, Clone)]
pub struct ParsingContext {
    pub report_type: ReportType,
    pub date: NaiveDate,
}

/// Report type enumeration
#[derive(Debug, Clone, Copy)]
pub enum ReportType {
    St1,
    St49,
}

impl ReportType {
    pub fn csv_prefix(&self) -> &'static str {
        match self {
            ReportType::St1 => "WELLS",
            ReportType::St49 => "SPUD",
        }
    }
}

/// Parsing error type
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Failed to parse date from content: {details}")]
    DateParse { details: String },
    
    #[error("Invalid field format: {details}")]
    FieldFormat { details: String },
    
    #[error("Missing required section: {section}")]
    MissingSection { section: String },
    
    #[error("File format error: {description}")]
    FileFormat { description: String },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
}

impl From<ParseError> for AppError {
    fn from(error: ParseError) -> Self {
        AppError::FileProcessing(error.to_string())
    }
}
