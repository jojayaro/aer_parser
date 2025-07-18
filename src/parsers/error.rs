//! Shared parsing error types
//!
//! This module provides comprehensive error handling for AER report parsing,
//! including context-rich error messages and recovery strategies.
//!
//! ## Error Categories
//!
//! - **Date Parsing**: Failed date extraction from report content
//! - **Field Format**: Invalid field formats or positions
//! - **Missing Sections**: Required report sections not found
//! - **File Format**: General file format issues
//! - **I/O Operations**: File system and I/O related errors
//!
//! ## Usage Example
//!
//! ```rust
//! use aer_parser::parsers::error::ParseError;
//!
//! let error = ParseError::DateParse {
//!     details: "Invalid date format".to_string()
//! };
//! ```

use thiserror::Error;

use crate::AppError;

/// Comprehensive parsing error type
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse date from content: {details}")]
    DateParse { details: String },
    
    #[error("Invalid field format at position {position}: {details}")]
    FieldFormat { position: usize, details: String },
    
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

/// Error recovery strategies
pub mod error_recovery {
    use super::*;

    /// Create a recoverable error with context
    pub fn recoverable_error(context: &str, details: &str) -> ParseError {
        ParseError::FileFormat {
            description: format!("{} - {}", context, details),
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(error: &ParseError) -> bool {
        matches!(
            error,
            ParseError::FieldFormat { .. } | ParseError::FileFormat { .. }
        )
    }
}
