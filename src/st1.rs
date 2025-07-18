//! ST-1 Well License Parser
//!
//! This module handles parsing of Alberta Energy Regulator ST-1 reports,
//! which contain information about well licenses issued, updated, or cancelled.
//!
//! ## Report Format
//!
//! ST-1 reports are text-based files with fixed-width fields containing:
//! - Well identification information
//! - License details
//! - Location and technical specifications
//! - Operator and contractor information
//!
//! ## Usage
//!
//! ```rust
//! use aer_parser::st1;
//!
//! // Process a single file
//! let date = st1::process_file("WELLS0102", "TXT", "CSV").await?;
//!
//! // Process all files in a folder
//! st1::process_folder("TXT", "CSV").await?;
//! ```

use crate::parsers::common::{date_utils, file_ops, trim_and_remove_empty_lines, write_csv_records};
use crate::parsers::error::ParseError;
use crate::AppError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Well license information from ST-1 reports
#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    /// Date the license was issued
    pub date: String,
    
    /// Well name as specified in the license
    pub well_name: String,
    
    /// AER license number
    pub licence_number: String,
    
    /// Mineral rights information
    pub mineral_rights: String,
    
    /// Ground elevation in meters
    pub ground_elevation: String,
    
    /// Unique identifier for the well
    pub unique_identifier: String,
    
    /// Surface coordinates (latitude/longitude)
    pub surface_coordinates: String,
    
    /// AER field centre designation
    pub aer_field_centre: String,
    
    /// Projected drilling depth in meters
    pub projected_depth: String,
    
    /// AER classification code
    pub aer_classification: String,
    
    /// Field name
    pub field: String,
    
    /// Terminating zone information
    pub terminating_zone: String,
    
    /// Type of drilling operation
    pub drilling_operation: String,
    
    /// Purpose of the well
    pub well_purpose: String,
    
    /// Well type classification
    pub well_type: String,
    
    /// Primary substance (oil, gas, etc.)
    pub substance: String,
    
    /// Licensee company name
    pub licensee: String,
    
    /// Surface location description
    pub surface_location: String,
}

/// Extract license data lines from ST1 report content
///
/// # Arguments
/// * `lines` - Reference to vector of file lines
///
/// # Returns
/// Vector of license data lines or parsing error
///
/// # Example
/// ```rust
/// let lines = vec!["WELL NAME".to_string(), "LICENCE NUMBER".to_string()];
/// let license_lines = extract_licences_lines(&lines)?;
/// ```
fn extract_licences_lines(lines: &[String]) -> Result<Vec<String>, ParseError> {
    let mut licences_lines: Vec<String> = Vec::new();
    let mut start_data_index: Option<usize> = None;

    // Find the start of the "WELL LICENCES ISSUED" data block
    for (i, line) in lines.iter().enumerate() {
        if line.contains("WELL NAME") && line.contains("LICENCE NUMBER") {
            // The actual data starts 6 lines after this header
            start_data_index = Some(i + 6);
            break;
        }
    }

    let start = start_data_index.ok_or_else(|| {
        ParseError::MissingSection {
            section: "WELL LICENCES ISSUED".to_string(),
        }
    })?;

    // Find the end of the data block
    let end = lines.iter().enumerate().skip(start)
        .find_map(|(i, line)| {
            if line.contains("WELL LICENCES UPDATED")
                || line.contains("WELL LICENCES CANCELLED")
                || line.contains("AMENDMENTS OF WELL LICENCES")
                || line.contains("END OF WELL LICENCES DAILY LIST")
                || line.contains("TOTAL")
                || line.contains("PAGE")
                || line.contains("WELL NAME AND U.I.D.")
                || line.contains("-------------------- END OF WELL LICENCES DAILY LIST")
            {
                Some(i)
            } else {
                None
            }
        })
        .unwrap_or(lines.len());

    // Extract valid data lines
    for line in lines.iter().take(end).skip(start) {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.contains("--------------------------------------------------------------------------------------------")
            && !trimmed.contains("TOTAL")
            && !trimmed.contains("PAGE")
            && !trimmed.contains("WELL NAME AND U.I.D.")
            && !trimmed.contains("END OF WELL LICENCES DAILY LIST")
            && !trimmed.contains("-------------------- END OF WELL LICENCES DAILY LIST")
            && trimmed.len() > 20
        {
            licences_lines.push(line.to_string());
        }
    }

    Ok(licences_lines)
}

/// Extract license data from parsed lines using fixed-width field positions
///
/// # Arguments
/// * `lines` - Vector of license data lines
/// * `date` - Report date for all licenses
///
/// # Returns
/// Vector of License structs
///
/// # Field Positions
/// - Line 0: well_name (0-37), licence_number (37-47), mineral_rights (47-68), ground_elevation (68+)
/// - Line 1: unique_identifier (0-37), surface_coordinates (37-47), aer_field_centre (47-68), projected_depth (68+)
/// - Line 2: aer_classification (0-37), field (37-68), terminating_zone (68+)
/// - Line 3: drilling_operation (0-37), well_purpose (37-47), well_type (47-68), substance (68+)
/// - Line 4: licensee (0-68), surface_location (68+)
fn extract_license(lines: Vec<String>, date: NaiveDate) -> Vec<License> {
    let mut licences: Vec<License> = Vec::new();
    
    for chunk in lines.chunks(5) {
        if chunk.len() == 5 {
            let line0 = &chunk[0];
            let line1 = &chunk[1];
            let line2 = &chunk[2];
            let line3 = &chunk[3];
            let line4 = &chunk[4];

            licences.push(License {
                date: date.to_string(),
                well_name: line0.get(0..37).unwrap_or("").trim().to_string(),
                licence_number: line0.get(37..47).unwrap_or("").trim().to_string(),
                mineral_rights: line0.get(47..68).unwrap_or("").trim().to_string(),
                ground_elevation: line0.get(68..).unwrap_or("").trim().to_string(),
                unique_identifier: line1.get(0..37).unwrap_or("").trim().to_string(),
                surface_coordinates: line1.get(37..47).unwrap_or("").trim().to_string(),
                aer_field_centre: line1.get(47..68).unwrap_or("").trim().to_string(),
                projected_depth: line1.get(68..).unwrap_or("").trim().to_string(),
                aer_classification: line2.get(0..37).unwrap_or("").trim().to_string(),
                field: line2.get(37..68).unwrap_or("").trim().to_string(),
                terminating_zone: line2.get(68..).unwrap_or("").trim().to_string(),
                drilling_operation: line3.get(0..37).unwrap_or("").trim().to_string(),
                well_purpose: line3.get(37..47).unwrap_or("").trim().to_string(),
                well_type: line3.get(47..68).unwrap_or("").trim().to_string(),
                substance: line3.get(68..).unwrap_or("").trim().to_string(),
                licensee: line4.get(0..68).unwrap_or("").trim().to_string(),
                surface_location: line4.get(68..).unwrap_or("").trim().to_string(),
            });
        }
    }
    licences
}

/// Process a single ST1 file and convert to CSV
///
/// # Arguments
/// * `filename_stem` - Base filename without extension (e.g., "WELLS0102")
/// * `txt_input_dir` - Directory containing input .TXT files
/// * `csv_output_dir` - Directory for output .CSV files
///
/// # Returns
/// The parsed date from the report
///
/// # Example
/// ```rust
/// let date = st1::process_file("WELLS0102", "TXT", "CSV").await?;
/// ```
pub async fn process_file(
    filename_stem: &str,
    txt_input_dir: &str,
    csv_output_dir: &str,
) -> Result<NaiveDate, AppError> {
    let filename = format!("{}/{}.TXT", txt_input_dir, filename_stem);
    let content = file_ops::read_file_content(&filename)?;
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let lines_trimmed = trim_and_remove_empty_lines(lines);

    let extracted_date = date_utils::extract_st1_date(&lines_trimmed)
        .map_err(|e| AppError::FileProcessing(e))?;

    let licences_lines = extract_licences_lines(&lines_trimmed)?;
    let licences_lines_trimmed = trim_and_remove_empty_lines(licences_lines);
    let licences = extract_license(licences_lines_trimmed, extracted_date);

    if !licences.is_empty() {
        let output_path = Path::new(csv_output_dir);
        write_csv_records(&licences, output_path, "WELLS", extracted_date)?;
    }

    Ok(extracted_date)
}

/// Process all ST1 files in a folder
///
/// # Arguments
/// * `folder_path` - Directory containing ST1 .TXT files
/// * `csv_output_dir` - Directory for output .CSV files
///
/// # Returns
/// Result indicating success or error
///
/// # Example
/// ```rust
/// st1::process_folder("TXT", "CSV").await?;
/// ```
pub async fn process_folder(folder_path: &str, csv_output_dir: &str) -> Result<(), AppError> {
    use crate::utils::process_folder_generic;
    
    let csv_output_dir = csv_output_dir.to_string();
    let folder_path_clone = folder_path.to_string();
    
    process_folder_generic(folder_path, "WELLS", move |filename_str| {
        let csv_output_dir = csv_output_dir.clone();
        let filename_path = PathBuf::from(&filename_str);
        let filename_stem = filename_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let folder_path_clone = folder_path_clone.clone();
        
        Box::pin(async move {
            let _ = process_file(&filename_stem, &folder_path_clone, &csv_output_dir).await;
            Ok(())
        })
    })
    .await
}
