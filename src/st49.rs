use crate::AppError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::utils::{open_file_lines, process_folder_generic};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpudData {
    pub date: String,
    pub well_id: String,
    pub well_name: String,
    pub licence: String,
    pub contractor_ba_id: String,
    pub contractor_name: String,
    pub rig_number: String,
    pub activity_date: String,
    pub field_centre: String,
    pub ba_id: String,
    pub licensee: String,
    pub new_projected_total_depth: String,
    pub activity_type: String,
}

fn extract_data_and_separator(lines: &[String]) -> (Vec<String>, Option<String>) {
    let mut data_lines = Vec::new();
    let mut separator_line = None;
    let mut start_index = None;
    let mut end_index = None;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("------") {
            start_index = Some(i + 1);
            separator_line = Some(line.clone());
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

    (data_lines, separator_line)
}

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

fn extract_spud_data(lines: Vec<String>, date: String, separator: &str) -> Vec<SpudData> {
    let mut spud_data_list: Vec<SpudData> = Vec::new();
    let boundaries = get_field_boundaries(separator);

    for line in lines {
        let get_field = |index: usize| -> String {
            if let Some(&(start, end)) = boundaries.get(index) {
                line.get(start..end).unwrap_or("").trim().to_string()
            } else {
                "".to_string()
            }
        };

        spud_data_list.push(SpudData {
            date: date.clone(),
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
            activity_type: line.get(boundaries.get(10).map_or(line.len(), |b| b.1)..).unwrap_or("").trim().to_string(),
        });
    }

    spud_data_list
}

fn write_spud_data_to_csv(spud_data: Vec<SpudData>, filename: &str, csv_output_dir: &str) -> Result<(), AppError> {
    if spud_data.is_empty() {
        return Ok(());
    }

    let output_filename = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b'|')
        .from_path(format!("{}/{}.csv", csv_output_dir, output_filename))?;
    for data in spud_data {
        wtr.serialize(data)?;
    }
    wtr.flush()?;
    Ok(())
}

fn extract_date(lines: &[String]) -> Result<String, AppError> {
    let date_line = lines.get(1).ok_or_else(|| AppError::FileProcessing("Date line not found".to_string()))?;
    let date_str = date_line.split_whitespace().skip(2).take(3).collect::<Vec<&str>>().join(" ");
    let parsed_date = NaiveDate::parse_from_str(&date_str, "%d %B %Y")?;
    Ok(parsed_date.format("%Y-%m-%d").to_string())
}

pub async fn process_file(filename: &str, csv_output_dir: &str) -> Result<(), AppError> {
    let lines = open_file_lines(filename)?;
    
    let formatted_date = extract_date(&lines)?;

    let (spud_data_lines, separator_line) = extract_data_and_separator(&lines);
    if let Some(separator) = separator_line {
        let spud_data = extract_spud_data(spud_data_lines, formatted_date, &separator);
        if !spud_data.is_empty() {
            write_spud_data_to_csv(spud_data, filename, csv_output_dir)?;
        }
    } else {
        return Err(AppError::FileProcessing("Separator line not found in file".to_string()));
    }

    Ok(())
}

pub async fn process_folder(folder_path: &str, csv_output_dir: &str) -> Result<(), AppError> {
    let csv_output_dir = csv_output_dir.to_string();
    process_folder_generic(folder_path, "SPUD", move |filename_str| {
        let csv_output_dir = csv_output_dir.clone();
        let filename_owned = filename_str.to_string();
        Box::pin(async move { process_file(&filename_owned, &csv_output_dir).await })
    }).await
}
