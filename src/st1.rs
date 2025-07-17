use crate::AppError;
use chrono::NaiveDate;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct License {
    date: String,
    well_name: String,
    licence_number: String,
    mineral_rights: String,
    ground_elevation: String,
    unique_identifier: String,
    surface_coordinates: String,
    aer_field_centre: String,
    projected_depth: String,
    aer_classification: String,
    field: String,
    terminating_zone: String,
    drilling_operation: String,
    well_purpose: String,
    well_type: String,
    substance: String,
    licensee: String,
    surface_location: String,
}

fn open_file_lines(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let content = BufReader::new(file);
    let lines: Vec<String> = content.lines().collect::<Result<_, _>>()?;
    Ok(lines)
}

fn trim_and_remove_empty_lines(lines: Vec<String>) -> Vec<String> {
    lines.into_iter().filter(|line| !line.trim().is_empty()).map(|line| line.trim().to_string()).collect()
}

fn extract_licences_lines(lines: &[String]) -> Result<Vec<String>, AppError> {
    let mut licences_lines: Vec<String> = Vec::new();

    let mut start_index = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("WELL NAME") && line.contains("LICENCE NUMBER") {
            start_index = i + 6; // 6 lines after the header line is where the first license starts
            break;
        }
    }

    let mut end_index = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if i > start_index && (line.contains("WELL LICENCES CANCELLED") || line.contains("AMENDMENTS OF WELL LICENCES") || line.contains("END OF WELL LICENCES DAILY LIST")) {
            end_index = i;
            break;
        }
    }

    for i in start_index..end_index {
        licences_lines.push(lines[i].to_string());
    }

    Ok(licences_lines)
}

fn get_field<'a>(line: &'a str, start: usize, end: usize) -> Option<&'a str> {
    line.get(start..end).map(|s| s.trim())
}

fn extract_license(lines: Vec<String>, date: String) -> Vec<License> {
    let mut licences: Vec<License> = Vec::new();
    for chunk in lines.chunks(5) {
        if chunk.len() == 5 {
            let line0 = &chunk[0];
            let line1 = &chunk[1];
            let line2 = &chunk[2];
            let line3 = &chunk[3];
            let line4 = &chunk[4];

            licences.push(License {
                date: date.clone(),
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

fn write_licence_to_csv(licences: Vec<License>, filename: &str) -> Result<(), AppError> {
    let output_filename = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    
    let mut wtr = csv::Writer::from_path(format!("CSV/{}.csv", output_filename))?;
    for licence in licences {
        wtr.serialize(licence)?;
    }
    wtr.flush()?;
    Ok(())
}



fn extract_date(lines: &[String]) -> Result<String, AppError> {
    let date_line = lines
        .iter()
        .find(|line| line.contains("DATE"))
        .ok_or_else(|| AppError::FileProcessing("No date line found in file".to_string()))?;

    let date_str = date_line.trim().get(6..).ok_or_else(|| {
        AppError::FileProcessing("Could not extract date string from line".to_string())
    })?;

    let parsed_date = NaiveDate::parse_from_str(date_str, "%d %B %Y")?;
    Ok(parsed_date.format("%Y-%m-%d").to_string())
}

pub async fn process_file(filename: &str) -> Result<(), AppError> {
    let lines = open_file_lines(filename)?;
    let lines_trimmed = trim_and_remove_empty_lines(lines);

    let formatted_date = extract_date(&lines_trimmed)?;

    let licences_lines = extract_licences_lines(&lines_trimmed)?;
    let licences_lines_trimmed = trim_and_remove_empty_lines(licences_lines);
    log::info!("Extracted and trimmed licences_lines: {:#?}", licences_lines_trimmed);
    let licences = extract_license(licences_lines_trimmed, formatted_date);
    log::info!("Extracted licences: {:#?}", licences);
    write_licence_to_csv(licences, filename)?;
    Ok(())
}

pub async fn process_folder(folder_path: &str) -> Result<(), AppError> {
    let folder = Path::new(folder_path);
    if !folder.is_dir() {
        return Err(AppError::FileProcessing(format!(
            "{} is not a valid directory",
            folder_path
        )));
    }

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.to_str() {
                if filename.contains("WELLS") && filename.ends_with(".TXT") {
                    info!("Processing file: {}", filename);
                    if let Err(e) = process_file(filename).await {
                        warn!("Failed to process file {}: {}", filename, e);
                    }
                }
            }
        }
    }
    Ok(())
}
