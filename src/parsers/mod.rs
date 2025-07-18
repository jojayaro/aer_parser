//! Shared parsing infrastructure for AER reports
//!
//! This module provides common utilities and traits for parsing
//! ST-1 and ST-49 reports, reducing code duplication and improving
//! maintainability.
//!
//! ## Architecture Overview
//!
//! The parsing infrastructure is built around shared utilities that
//! handle common operations across different report types:
//!
//! - **Common utilities**: File operations, date parsing, CSV writing
//! - **Error handling**: Context-rich error messages with recovery strategies
//! - **Memory optimization**: Buffered reading and streaming operations
//!
//! ## Usage Example
//!
//! ```rust
//! use aer_parser::parsers::common::{date_utils, file_ops, write_csv_records};
//! use chrono::NaiveDate;
//!
//! // Read and parse a file
//! let content = file_ops::read_file_content("path/to/file.txt")?;
//! let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
//! let date = date_utils::extract_st1_date(&lines)?;
//! ```

pub mod common;
pub mod error;
pub mod traits;

pub use common::*;
pub use traits::*;
