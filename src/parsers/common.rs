//! Common utilities for report parsing
//!
//! This module provides shared functionality for parsing AER reports,
//! including file operations, date parsing, CSV writing, and text processing.
//!
//! ## Features
//!
//! - **File Operations**: Efficient file reading with buffered I/O
//! - **Date Parsing**: Specialized date extraction for ST1/ST49 formats
//! - **CSV Writing**: Streaming CSV output with proper formatting
//! - **Text Processing**: Common text manipulation utilities
//!
//! ## Example Usage
//!
//! ```rust
//! use aer_parser::parsers::common::{date_utils, file_ops, write_csv_records};
//! use chrono::NaiveDate;
//! use std::path::Path;
//!
//! // Read a file
//! let content = file_ops::read_file_content("data.txt")?;
//! let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
//!
//! // Parse date
//! let date = date_utils::extract_st1_date(&lines)?;
//!
//! // Write CSV
//! let records = vec![/* your data */];
//! write_csv_records(&records, Path::new("output"), "report", date)?;
//! ```

use std::io::Read;
use std::path::Path;

use chrono::NaiveDate;
use serde::Serialize;

use crate::AppError;

/// Trim and remove empty lines from content
///
/// # Arguments
/// * `lines` - Vector of strings to process
///
/// # Returns
/// Vector of non-empty, trimmed strings
///
/// # Example
/// ```rust
/// let lines = vec!["  Hello  ".to_string(), "".to_string(), "World".to_string()];
/// let cleaned = trim_and_remove_empty_lines(lines);
/// assert_eq!(cleaned, vec!["Hello", "World"]);
/// ```
pub fn trim_and_remove_empty_lines(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

/// Write records to CSV file with standardized naming
///
/// # Arguments
/// * `records` - Slice of serializable records to write
/// * `output_path` - Directory path for output
/// * `filename_prefix` - Prefix for filename (e.g., "WELLS", "SPUD")
/// * `report_date` - Date to include in filename
///
/// # Returns
/// Result indicating success or error
///
/// # Example
/// ```rust
/// #[derive(serde::Serialize)]
/// struct Record { date: String, value: String }
/// let records = vec![Record { date: "2024-01-01".to_string(), value: "test".to_string() }];
/// write_csv_records(&records, Path::new("output"), "WELLS", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())?;
/// ```
pub fn write_csv_records<T: Serialize>(
    records: &[T],
    output_path: &Path,
    filename_prefix: &str,
    report_date: NaiveDate,
) -> Result<(), AppError> {
    if records.is_empty() {
        return Ok(());
    }

    let output_filename = format!("{}_{}.csv", report_date.format("%Y%m%d"), filename_prefix);
    let full_path = output_path.join(output_filename);

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b',')
        .from_path(full_path)?;
    
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    
    Ok(())
}

/// Common date extraction utilities
pub mod date_utils {
    use super::*;

    /// Extract date from ST1 format
    ///
    /// # Arguments
    /// * `lines` - Reference to vector of file lines
    ///
    /// # Returns
    /// Parsed NaiveDate or error message
    ///
    /// # Example
    /// ```rust
    /// let lines = vec!["DATE: 02 January 2024".to_string()];
    /// let date = date_utils::extract_st1_date(&lines)?;
    /// ```
    pub fn extract_st1_date(lines: &[String]) -> Result<NaiveDate, String> {
        let date_line = lines
            .iter()
            .find(|line| line.contains("DATE"))
            .ok_or_else(|| "No date line found in file".to_string())?;

        let date_str = date_line.trim().get(6..).ok_or_else(|| {
            "Could not extract date string from line".to_string()
        })?;

        NaiveDate::parse_from_str(date_str, "%d %B %Y")
            .map_err(|e| format!("Failed to parse date: {}", e))
    }

    /// Extract date from ST49 format
    ///
    /// # Arguments
    /// * `lines` - Reference to vector of file lines
    ///
    /// # Returns
    /// Parsed NaiveDate or error message
    ///
    /// # Example
    /// ```rust
    /// let lines = vec!["AER DAILY SPUD REPORT 02 January 2024".to_string()];
    /// let date = date_utils::extract_st49_date(&lines)?;
    /// ```
    pub fn extract_st49_date(lines: &[String]) -> Result<NaiveDate, String> {
        let date_line = lines
            .get(1)
            .ok_or_else(|| "Date line not found".to_string())?;
        
        let date_str = date_line
            .split_whitespace()
            .skip(2)
            .take(3)
            .collect::<Vec<&str>>()
            .join(" ");
        
        NaiveDate::parse_from_str(&date_str, "%d %B %Y")
            .map_err(|e| format!("Failed to parse date: {}", e))
    }
}

/// Common file operations
pub mod file_ops {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    use super::*;

    /// Read file content as string
    ///
    /// # Arguments
    /// * `filename` - Path to file to read
    ///
    /// # Returns
    /// File content as string or I/O error
    ///
    /// # Example
    /// ```rust
    /// let content = file_ops::read_file_content("data.txt")?;
    /// ```
    pub fn read_file_content(filename: &str) -> Result<String, std::io::Error> {
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Read file lines efficiently
    ///
    /// # Arguments
    /// * `filename` - Path to file to read
    ///
    /// # Returns
    /// Vector of lines or I/O error
    pub fn read_file_lines(filename: &str) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        reader.lines().collect()
    }
}

/// Common error handling utilities
pub mod error_utils {
    /// Create context-rich error messages
    ///
    /// # Arguments
    /// * `context` - Context description
    /// * `details` - Specific error details
    ///
    /// # Returns
    /// Formatted error message
    pub fn create_parse_error(context: &str, details: &str) -> String {
        format!("{}: {}", context, details)
    }

    /// Validate file extension
    ///
    /// # Arguments
    /// * `filename` - File path to validate
    /// * `expected` - Expected extension (e.g., "txt", "csv")
    ///
    /// # Returns
    /// Result indicating valid extension or error
    pub fn validate_file_extension(filename: &str, expected: &str) -> Result<(), String> {
        if !filename.to_lowercase().ends_with(&expected.to_lowercase()) {
            return Err(format!(
                "Invalid file extension. Expected: {}, Got: {}",
                expected,
                std::path::Path::new(filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("none")
            ));
        }
        Ok(())
    }
}
