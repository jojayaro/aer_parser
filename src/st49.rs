use std::fs::{self, File};
use std::io::{BufReader, BufRead};
use std::path::Path;
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use crate::AppError;

#[derive(Debug, Serialize, Deserialize)]
struct SpudData {
    date: String,
    well_id: String,
    well_name: String,
    licence: String,
    contractor_ba_id: String,
    contractor_name: String,
    rig_number: String,
    activity_date: String,
    field_centre: String,
    ba_id: String,
    licensee: String,
    new_projected_total_depth: String,
    activity_type: String,
}

fn open_file_lines(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let content = BufReader::new(file);
    let lines: Vec<String> = content.lines().collect::<Result<_, _>>()?;
    Ok(lines)
}

fn extract_spud_data_lines(lines: &[String]) -> Vec<String> {
    let mut data_lines = Vec::new();
    let mut start_index = None;
    let mut end_index = None;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("------") {
            start_index = Some(i + 1);
        }
        if line.contains("TOTAL  -") {
            end_index = Some(i);
            break;
        }
    }

    if let (Some(start), Some(end)) = (start_index, end_index) {
        for i in start..end {
            if !lines[i].trim().is_empty() {
                data_lines.push(lines[i].to_string());
            }
        }
    }

    data_lines
}

fn extract_spud_data(lines: Vec<String>, date: String) -> Vec<SpudData> {
    let mut spud_data_list: Vec<SpudData> = Vec::new();

    for line in lines {
        spud_data_list.push(SpudData {
            date: date.clone(),
            well_id: line.get(0..19).unwrap_or("").trim().to_string(),
            well_name: line.get(19..52).unwrap_or("").trim().to_string(),
            licence: line.get(52..60).unwrap_or("").trim().to_string(),
            contractor_ba_id: line.get(60..66).unwrap_or("").trim().to_string(),
            contractor_name: line.get(66..99).unwrap_or("").trim().to_string(),
            rig_number: line.get(99..105).unwrap_or("").trim().to_string(),
            activity_date: line.get(105..116).unwrap_or("").trim().to_string(),
            field_centre: line.get(116..133).unwrap_or("").trim().to_string(),
            ba_id: line.get(133..139).unwrap_or("").trim().to_string(),
            licensee: line.get(139..172).unwrap_or("").trim().to_string(),
            new_projected_total_depth: line.get(172..178).unwrap_or("").trim().to_string(),
            activity_type: line.get(178..).unwrap_or("").trim().to_string(),
        });
    }

    spud_data_list
}

fn write_spud_data_to_csv(spud_data: Vec<SpudData>, filename: &str) -> Result<(), AppError> {
    let output_filename = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let mut wtr = csv::Writer::from_path(format!("CSV/{}.csv", output_filename))?;
    for data in spud_data {
        wtr.serialize(data)?;
    }
    wtr.flush()?;
    Ok(())
}

pub async fn process_file(filename: &str) -> Result<(), AppError> {
    let lines = open_file_lines(filename)?;
    
    let date_line = lines.get(1).ok_or_else(|| AppError::FileProcessing("Date line not found".to_string()))?;
    let date_str = date_line.split_whitespace().skip(2).take(3).collect::<Vec<&str>>().join(" ");
    let parsed_date = NaiveDate::parse_from_str(&date_str, "%d %B %Y")?;
    let formatted_date = parsed_date.format("%Y-%m-%d").to_string();

    let spud_data_lines = extract_spud_data_lines(&lines);
    let spud_data = extract_spud_data(spud_data_lines, formatted_date);
    write_spud_data_to_csv(spud_data, filename)?;
    Ok(())
}

pub async fn process_folder(folder_path: &str) -> Result<(), AppError> {
    let folder = Path::new(folder_path);
    if !folder.is_dir() {
        return Err(AppError::FileProcessing(format!("{} is not a valid directory", folder_path)));
    }

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("TXT") {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with("SPUD") {
                    println!("Processing file: {}", path.display());
                    process_file(path.to_str().unwrap_or_default()).await?;
                }
            }
        }
    }
    Ok(())
}
