//! ST-49 Spud Report Parser
//!
//! This module handles parsing of Alberta Energy Regulator ST-49 reports,
//! which contain information about wells that have been spudded (drilling commenced).
//!
//! ## Report Format
//!
//! ST-49 reports are text-based files with fixed-width fields containing:
//! - Well identification and spud information
//! - Contractor and rig details
//! - Activity dates and depths
//! - Location and operator information
//!
//! ## Usage
//!
//! ```rust
//! use aer_parser::st49;
//!
//! // Process a single file
//! let date = st49::process_file("SPUD0101", "TXT", "CSV").await?;
//!
//! // Process all files in a folder
//! st49::process_folder("TXT", "CSV").await?;
//! ```

use crate::parsers::common::{date_utils, file_ops, trim_and_remove_empty_lines, write_csv_records};
use crate::parsers::error::ParseError;
use crate::AppError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Spud information from ST-49 reports
#[derive(Debug, Serialize, Deserialize)]
pub struct SpudData {
    /// Date of the spud report
    pub date: String,
    
    /// Well identification number
    pub well_id: String,
    
    /// Well name
    pub well_name: String,
    
    /// License number
    pub licence: String,
    
    /// Contractor BA ID
    pub contractor_ba_id: String,
    
    /// Contractor company name
    pub contractor_name: String,
    
    /// Rig identification number
    pub rig_number: String,
    
    /// Date of spudding activity
    pub activity_date: String,
    
    /// AER field centre designation
    pub field_centre: String,
    
    /// BA ID of the operator
    pub ba_id: String,
    
    /// Licensee company name
    pub licensee: String,
    
    /// New projected total depth in meters
    pub new_projected_total_depth: String,
    
    /// Type of activity (SPUD, RIG RELEASE, etc.)
    pub activity_type: String,
}

/// Extract data and separator line from ST49 report content
///
/// # Arguments
/// * `lines` - Reference to vector of file lines
///
/// # Returns
/// Tuple of (data lines, separator line) or parsing error
///
/// # Example
/// ```rust
/// let lines = vec!["AER DAILY SPUD REPORT".to_string(), "------".to_string()];
/// let (data_lines, separator) = extract_data_and_separator(&lines)?;
/// ```
fn extract_data_and_separator(lines: &[String]) -> Result<(Vec<String>, String), ParseError> {
    let mut all_data_lines = Vec::new();
    let mut separator_line = None;
    let mut in_data_block = false;

    for line in lines.iter() {
        if line.contains("------") {
            separator_line = Some(line.clone());
            in_data_block = true;
            continue;
        }

        if in_data_block {
            // Filter out header-like lines
            if (line.contains("BA ID") && line.contains("NAME") && line.contains("NUMBER"))
                || (line.starts_with("20") && line.contains("BA ID") && line.contains("NAME") && line.contains("NUMBER"))
            {
                continue;
            }

            // Filter out footer/summary lines
            if line.contains("Report Number:")
                || line.contains("Run Date:")
                || line.contains("For the Notification Period")
                || line.contains("TOTAL  -")
                || line.contains("WELL ID")
                || line.contains("PAGE")
                || line.contains("Report Number: ST-4")
            {
                in_data_block = false;
                continue;
            }

            if !line.trim().is_empty()
                && !line.contains("AER DAILY SPUD REPORT")
                && line.len() > 10
            {
                all_data_lines.push(line.to_string());
            }
        }
    }

    let separator = separator_line.ok_or_else(|| {
        ParseError::MissingSection {
            section: "separator line".to_string(),
        }
    })?;

    Ok((all_data_lines, separator))
}

/// Get field boundaries from separator line
///
/// # Arguments
/// * `separator` - The separator line containing field boundaries
///
/// # Returns
/// Vector of (start, end) positions for each field
///
/// # Example
/// ```rust
/// let separator = "------    ------    ------";
/// let boundaries = get_field_boundaries(separator);
/// ```
fn get_field_boundaries(separator: &str) -> Vec<(usize, usize)> {
    let mut boundaries = Vec::new();
    let mut start = 0;
    
    for (i, char) in separator.char_indices() {
        if char.is_whitespace() {
            if i > start {
                boundaries.push((start, i));
            }
            start = i + 1;
        }
    }
    boundaries.push((start, separator.len()));
    boundaries
}

/// Extract spud data from parsed lines using field boundaries
///
/// # Arguments
/// * `lines` - Vector of data lines
/// * `date` - Report date for all records
/// * `separator` - Separator line for field boundary detection
///
/// # Returns
/// Vector of SpudData structs
///
/// # Field Positions
/// - Field 0: well_id
/// - Field 1: well_name
/// - Field 2: licence
/// - Field 3: contractor_ba_id
/// - Field 4: contractor_name
/// - Field 5: rig_number
/// - Field 6: activity_date
/// - Field 7: field_centre
/// - Field 8: ba_id
/// - Field 9: licensee
/// - Field 10: new_projected_total_depth
/// - Remaining: activity_type
fn extract_spud_data(lines: Vec<String>, date: NaiveDate, separator: &str) -> Vec<SpudData> {
    let mut spud_data_list: Vec<SpudData> = Vec::new();
    let boundaries = get_field_boundaries(separator);

    for line in lines {
        let get_field = |index: usize| -> String {
            boundaries.get(index)
                .map(|(start, end)| line.get(*start..*end).unwrap_or("").trim().to_string())
                .unwrap_or_default()
        };

        spud_data_list.push(SpudData {
            date: date.to_string(),
            well_id: get_field(0),
            well_name: get_field(1),
            licence: get_field(2),
            contractor_ba_id: get_field(3),
            contractor_name: get_field(4),
            rig_number: get_field(5),
            activity_date: get_field(6),
            field_centre: get_field(7),
            ba_id: get_field(8),
            licensee: get_field(9),
            new_projected_total_depth: get_field(10),
            activity_type: line
                .get(boundaries.get(10).map_or(line.len(), |b| b.1)..)
                .unwrap_or("")
                .trim()
                .to_string(),
        });
    }

    spud_data_list
}

/// Process a single ST49 file and convert to CSV
///
/// # Arguments
/// * `filename_stem` - Base filename without extension (e.g., "SPUD0101")
/// * `txt_input_dir` - Directory containing input .TXT files
/// * `csv_output_dir` - Directory for output .CSV files
///
/// # Returns
/// The parsed date from the report
///
/// # Example
/// ```rust
/// let date = st49::process_file("SPUD0101", "TXT", "CSV").await?;
/// ```
pub async fn process_file(
    filename_stem: &str,
    txt_input_dir: &str,
    csv_output_dir: &str,
) -> Result<NaiveDate, AppError> {
    let filename = format!("{}/{}.txt", txt_input_dir, filename_stem);
    let content = file_ops::read_file_content(&filename)?;
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let lines_trimmed = trim_and_remove_empty_lines(lines);

    let extracted_date = date_utils::extract_st49_date(&lines_trimmed)
        .map_err(|e| AppError::FileProcessing(e))?;

    let (spud_data_lines, separator_line) = extract_data_and_separator(&lines_trimmed)?;
    let spud_data = extract_spud_data(spud_data_lines, extracted_date, &separator_line);

    if !spud_data.is_empty() {
        let output_path = Path::new(csv_output_dir);
        write_csv_records(&spud_data, output_path, "SPUD", extracted_date)?;
    }

    Ok(extracted_date)
}

/// Process all ST49 files in a folder
///
/// # Arguments
/// * `folder_path` - Directory containing ST49 .TXT files
/// * `csv_output_dir` - Directory for output .CSV files
///
/// # Returns
/// Result indicating success or error
///
/// # Example
/// ```rust
/// st49::process_folder("TXT", "CSV").await?;
/// ```
pub async fn process_folder(folder_path: &str, csv_output_dir: &str) -> Result<(), AppError> {
    use crate::utils::process_folder_generic;
    
    let csv_output_dir = csv_output_dir.to_string();
    let folder_path_clone = folder_path.to_string();
    
    process_folder_generic(folder_path, "SPUD", move |filename_str| {
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
