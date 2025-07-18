//! Simple unit tests for AER parser components
//!
//! This module contains basic unit tests that don't require public access to internal functions.

use aer_st1::parsers::common::{date_utils, file_ops, trim_and_remove_empty_lines};
use chrono::NaiveDate;

#[test]
fn test_trim_and_remove_empty_lines() {
    let input = vec![
        "  Hello  ".to_string(),
        "".to_string(),
        "World".to_string(),
        "   ".to_string(),
        "Test".to_string(),
    ];
    let expected = vec!["Hello".to_string(), "World".to_string(), "Test".to_string()];
    let result = trim_and_remove_empty_lines(input);
    assert_eq!(result, expected);
}

#[test]
fn test_extract_st1_date_valid() {
    let lines = vec![
        "ALBERTA ENERGY REGULATOR".to_string(),
        "DATE: 02 January 2024".to_string(),
    ];
    let result = date_utils::extract_st1_date(&lines);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), NaiveDate::from_ymd_opt(2024, 1, 2).unwrap());
}

#[test]
fn test_extract_st1_date_missing() {
    let lines = vec!["No date here".to_string()];
    let result = date_utils::extract_st1_date(&lines);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No date line found in file");
}

#[test]
fn test_extract_st49_date_valid() {
    // Skip this test for now - implementation needs adjustment
    let lines = vec![
        "AER DAILY SPUD REPORT".to_string(),
        "02 January 2024".to_string(),
    ];
    let result = date_utils::extract_st49_date(&lines);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), NaiveDate::from_ymd_opt(2024, 1, 2).unwrap());
}

#[test]
fn test_extract_st49_date_missing() {
    let lines = vec!["No date here".to_string()];
    let result = date_utils::extract_st49_date(&lines);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Date line not found");
}

#[test]
fn test_file_ops_read_file_content() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();
    
    let content = file_ops::read_file_content(file_path.to_str().unwrap());
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "Hello, World!");
}

#[test]
fn test_file_ops_read_file_lines() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Line 1\nLine 2\nLine 3").unwrap();
    
    let lines = file_ops::read_file_lines(file_path.to_str().unwrap());
    assert!(lines.is_ok());
    let lines = lines.unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "Line 1");
    assert_eq!(lines[1], "Line 2");
    assert_eq!(lines[2], "Line 3");
}
