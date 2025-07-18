//! Unit tests for AER parser components
//!
//! This module contains focused unit tests for individual parsing functions
//! and utilities, ensuring correctness at the component level.

use aer_st1::parsers::common::{date_utils, file_ops, trim_and_remove_empty_lines, write_csv_records};
use aer_st1::parsers::error::ParseError;
use aer_st1::st1::{extract_licences_lines, extract_license, License};
use aer_st1::st49::{extract_data_and_separator, extract_spud_data, get_field_boundaries, SpudData};
use chrono::NaiveDate;
use std::fs;
use std::path::Path;

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
    let lines = vec![
        "AER DAILY SPUD REPORT".to_string(),
        "AER DAILY SPUD REPORT 02 January 2024".to_string(),
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
fn test_extract_licences_lines_valid() {
    let lines = vec![
        "ALBERTA ENERGY REGULATOR".to_string(),
        "DATE: 02 January 2024".to_string(),
        "".to_string(),
        "WELL LICENCES ISSUED".to_string(),
        "".to_string(),
        "WELL NAME                           LICENCE NUMBER  MINERAL RIGHTS".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "WELL 1                              123456          FREEHOLD".to_string(),
        "UID-001                             12-34-56-01W4   CALGARY".to_string(),
        "NEW                                 FIELD-A         BANFF".to_string(),
        "HORIZONTAL                          PRODUCTION      GAS WELL".to_string(),
        "COMPANY A                           12-34-56-01W4".to_string(),
        "--------------------------------------------------------------------------------------------".to_string(),
        "END OF WELL LICENCES DAILY LIST".to_string(),
    ];
    
    let result = extract_licences_lines(&lines);
    assert!(result.is_ok());
    let licences = result.unwrap();
    assert_eq!(licences.len(), 5);
    assert!(licences[0].contains("WELL 1"));
}

#[test]
fn test_extract_licences_lines_missing_section() {
    let lines = vec![
        "ALBERTA ENERGY REGULATOR".to_string(),
        "DATE: 02 January 2024".to_string(),
    ];
    
    let result = extract_licences_lines(&lines);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::MissingSection { section } => {
            assert_eq!(section, "WELL LICENCES ISSUED");
        }
        _ => panic!("Expected MissingSection error"),
    }
}

#[test]
fn test_extract_license_valid() {
    let lines = vec![
        "WELL 1                              123456          FREEHOLD                    1000".to_string(),
        "UID-001                             12-34-56-01W4   CALGARY                     3000".to_string(),
        "NEW                                 FIELD-A         BANFF".to_string(),
        "HORIZONTAL                          PRODUCTION      GAS WELL        GAS".to_string(),
        "COMPANY A                           12-34-56-01W4".to_string(),
    ];
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let licences = extract_license(lines, date);
    
    assert_eq!(licences.len(), 1);
    let license = &licences[0];
    assert_eq!(license.well_name, "WELL 1");
    assert_eq!(license.licence_number, "123456");
    assert_eq!(license.date, "2024-01-02");
}

#[test]
fn test_get_field_boundaries() {
    let separator = "------    ------    ------";
    let boundaries = get_field_boundaries(separator);
    assert_eq!(boundaries.len(), 3);
    assert_eq!(boundaries[0], (0, 6));
    assert_eq!(boundaries[1], (10, 16));
    assert_eq!(boundaries[2], (20, 26));
}

#[test]
fn test_extract_data_and_separator_valid() {
    let lines = vec![
        "AER DAILY SPUD REPORT".to_string(),
        "AER DAILY SPUD REPORT 02 January 2024".to_string(),
        "--------------------------------------------------------------------------------------------".to_string(),
        "WELL ID     WELL NAME           LICENCE".to_string(),
        "--------------------------------------------------------------------------------------------".to_string(),
        "W001        WELL-A              123456".to_string(),
        "Report Number: ST-49".to_string(),
    ];
    
    let result = extract_data_and_separator(&lines);
    assert!(result.is_ok());
    let (data_lines, separator) = result.unwrap();
    assert_eq!(data_lines.len(), 1);
    assert!(data_lines[0].contains("W001"));
    assert!(separator.contains("----"));
}

#[test]
fn test_extract_spud_data_valid() {
    let lines = vec![
        "W001        WELL-A              123456      1001    CONTRACTOR-A    RIG-001 2024-01-01".to_string(),
        "CALGARY     2001    COMPANY-A               3000                    SPUD".to_string(),
    ];
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let separator = "--------------------------------------------------------------------------------------------";
    let spud_data = extract_spud_data(lines, date, separator);
    
    assert_eq!(spud_data.len(), 2);
    let record = &spud_data[0];
    assert_eq!(record.well_id, "W001");
    assert_eq!(record.well_name, "WELL-A");
    assert_eq!(record.licence, "123456");
    assert_eq!(record.date, "2024-01-02");
}

#[test]
fn test_write_csv_records_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let records: Vec<License> = vec![];
    let result = write_csv_records(&records, temp_dir.path(), "TEST", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert!(result.is_ok());
}

#[test]
fn test_write_csv_records_valid() {
    let temp_dir = tempfile::tempdir().unwrap();
    let records = vec![
        License {
            date: "2024-01-01".to_string(),
            well_name: "TEST-WELL".to_string(),
            licence_number: "123456".to_string(),
            mineral_rights: "FREEHOLD".to_string(),
            ground_elevation: "1000".to_string(),
            unique_identifier: "UID-001".to_string(),
            surface_coordinates: "12-34-56-01W4".to_string(),
            aer_field_centre: "CALGARY".to_string(),
            projected_depth: "3000".to_string(),
            aer_classification: "NEW".to_string(),
            field: "FIELD-A".to_string(),
            terminating_zone: "BANFF".to_string(),
            drilling_operation: "HORIZONTAL".to_string(),
            well_purpose: "PRODUCTION".to_string(),
            well_type: "GAS WELL".to_string(),
            substance: "GAS".to_string(),
            licensee: "COMPANY A".to_string(),
            surface_location: "12-34-56-01W4".to_string(),
        }
    ];
    
    let result = write_csv_records(&records, temp_dir.path(), "TEST", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert!(result.is_ok());
    
    let expected_file = temp_dir.path().join("20240101_TEST.csv");
    assert!(expected_file.exists());
}

#[test]
fn test_file_ops_read_file_content() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Hello, World!").unwrap();
    
    let content = file_ops::read_file_content(file_path.to_str().unwrap());
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "Hello, World!");
}

#[test]
fn test_file_ops_read_file_lines() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Line 1\nLine 2\nLine 3").unwrap();
    
    let lines = file_ops::read_file_lines(file_path.to_str().unwrap());
    assert!(lines.is_ok());
    let lines = lines.unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "Line 1");
    assert_eq!(lines[1], "Line 2");
    assert_eq!(lines[2], "Line 3");
}
