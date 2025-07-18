use crate::utils::{open_file_lines, process_folder_generic};
use crate::AppError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub date: String,
    pub well_name: String,
    pub licence_number: String,
    pub mineral_rights: String,
    pub ground_elevation: String,
    pub unique_identifier: String,
    pub surface_coordinates: String,
    pub aer_field_centre: String,
    pub projected_depth: String,
    pub aer_classification: String,
    pub field: String,
    pub terminating_zone: String,
    pub drilling_operation: String,
    pub well_purpose: String,
    pub well_type: String,
    pub substance: String,
    pub licensee: String,
    pub surface_location: String,
}

fn trim_and_remove_empty_lines(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn extract_licences_lines(lines: &[String]) -> Result<Vec<String>, AppError> {
    let mut licences_lines: Vec<String> = Vec::new();
    let mut start_data_index: Option<usize> = None;
    let mut end_data_index: Option<usize> = None;

    // Find the start of the "WELL LICENCES ISSUED" data block
    for (i, line) in lines.iter().enumerate() {
        if line.contains("WELL NAME") && line.contains("LICENCE NUMBER") {
            // The actual data starts 6 lines after this header
            start_data_index = Some(i + 6);
            break;
        }
    }

    if let Some(start) = start_data_index {
        // Find the end of the "WELL LICENCES ISSUED" data block
        // This is typically marked by the start of the next section or the end of the file
        for i in start..lines.len() {
            let line = &lines[i];
            if line.contains("WELL LICENCES UPDATED")
                || line.contains("WELL LICENCES CANCELLED")
                || line.contains("AMENDMENTS OF WELL LICENCES")
                || line.contains("END OF WELL LICENCES DAILY LIST")
            {
                end_data_index = Some(i);
                break;
            }
        }

        // If no specific end marker is found, the data goes to the end of the file
        let end = end_data_index.unwrap_or(lines.len());

        // Extract lines between start and end, ensuring they are not just empty or separator lines
        for i in start..end {
            let line = &lines[i];
            // Only add lines that are not empty and not separator lines
            if !line.trim().is_empty() && !line.contains("--------------------------------------------------------------------------------------------") {
                licences_lines.push(line.to_string());
            }
        }
    }

    Ok(licences_lines)
}

fn get_field<'a>(line: &'a str, start: usize, end: usize) -> Option<&'a str> {
    line.get(start..end).map(|s| s.trim())
}

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
                well_name: get_field(line0, 0, 37).unwrap_or("").to_string(),
                licence_number: get_field(line0, 37, 47).unwrap_or("").to_string(),
                mineral_rights: get_field(line0, 47, 68).unwrap_or("").to_string(),
                ground_elevation: get_field(line0, 68, line0.len()).unwrap_or("").to_string(),
                unique_identifier: get_field(line1, 0, 37).unwrap_or("").to_string(),
                surface_coordinates: get_field(line1, 37, 47).unwrap_or("").to_string(),
                aer_field_centre: get_field(line1, 47, 68).unwrap_or("").to_string(),
                projected_depth: get_field(line1, 68, line1.len()).unwrap_or("").to_string(),
                aer_classification: get_field(line2, 0, 37).unwrap_or("").to_string(),
                field: get_field(line2, 37, 68).unwrap_or("").to_string(),
                terminating_zone: get_field(line2, 68, line2.len()).unwrap_or("").to_string(),
                drilling_operation: get_field(line3, 0, 37).unwrap_or("").to_string(),
                well_purpose: get_field(line3, 37, 47).unwrap_or("").to_string(),
                well_type: get_field(line3, 47, 68).unwrap_or("").to_string(),
                substance: get_field(line3, 68, line3.len()).unwrap_or("").to_string(),
                licensee: get_field(line4, 0, 68).unwrap_or("").to_string(),
                surface_location: get_field(line4, 68, line4.len()).unwrap_or("").to_string(),
            });
        }
    }
    licences
}

fn write_licence_to_csv(
    licences: Vec<License>,
    filename: &str,
    csv_output_dir: &str,
) -> Result<(), AppError> {
    if licences.is_empty() {
        return Ok(());
    }

    let output_filename = Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or("output").to_string();

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b',')
        .from_path(format!("{}/{}.csv", csv_output_dir, output_filename))?;
    for licence in licences {
        wtr.serialize(licence)?;
    }
    wtr.flush()?;
    Ok(())
}

fn extract_date(lines: &[String]) -> Result<NaiveDate, AppError> {
    let date_line = lines
        .iter()
        .find(|line| line.contains("DATE"))
        .ok_or_else(|| AppError::FileProcessing("No date line found in file".to_string()))?;

    let date_str = date_line.trim().get(6..).ok_or_else(|| {
        AppError::FileProcessing("Could not extract date string from line".to_string())
    })?;

    let parsed_date = NaiveDate::parse_from_str(date_str, "%d %B %Y")?;
    Ok(parsed_date)
}

pub async fn process_file(filename: &str, csv_output_dir: &str) -> Result<NaiveDate, AppError> {
    let lines = open_file_lines(filename)?;
    let lines_trimmed = trim_and_remove_empty_lines(lines);

    let extracted_date = extract_date(&lines_trimmed)?;

    let licences_lines = extract_licences_lines(&lines_trimmed)?;
    let licences_lines_trimmed = trim_and_remove_empty_lines(licences_lines);
    log::debug!(
        "Extracted and trimmed licences_lines: {:#?}",
        licences_lines_trimmed
    );
    let licences = extract_license(licences_lines_trimmed, extracted_date);
    log::debug!("Extracted licences: {:#?}", licences);
    if !licences.is_empty() {
        write_licence_to_csv(licences, filename, csv_output_dir)?;
    }
    Ok(extracted_date)
}

pub async fn process_folder(folder_path: &str, csv_output_dir: &str) -> Result<(), AppError> {
    let csv_output_dir = csv_output_dir.to_string();
    process_folder_generic(folder_path, "WELLS", move |filename_str| {
        let csv_output_dir = csv_output_dir.clone();
        let filename_owned = filename_str.to_string();
        Box::pin(async move { 
            let _ = process_file(&filename_owned, &csv_output_dir).await; // Year is not used in process_folder
            Ok(()) 
        })
    })
    .await
}
