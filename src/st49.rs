use crate::utils::{open_file_lines, process_folder_generic};
use crate::AppError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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

fn trim_and_remove_empty_lines(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn extract_data_and_separator(lines: &[String]) -> (Vec<String>, Option<String>) {
    let mut all_data_lines = Vec::new();
    let mut current_separator_line = None;
    let mut in_data_block = false;

    for line in lines.iter() {
        if line.contains("------") {
            current_separator_line = Some(line.clone());
            in_data_block = true;
            continue;
        }

        if in_data_block {
            // Filter out header-like lines that appear within the data block
            if (line.contains("BA ID") && line.contains("NAME") && line.contains("NUMBER"))
                || (line.starts_with("20")
                    && line.contains("BA ID")
                    && line.contains("NAME")
                    && line.contains("NUMBER"))
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
            // Specific footer for ST49
            {
                in_data_block = false;
                continue;
            }
            if !line.trim().is_empty() && !line.contains("AER DAILY SPUD REPORT") && line.len() > 10
            {
                all_data_lines.push(line.to_string());
            }
        }
    }

    (all_data_lines, current_separator_line)
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

fn extract_spud_data(lines: Vec<String>, date: NaiveDate, separator: &str) -> Vec<SpudData> {
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

fn write_spud_data_to_csv(
    spud_data: Vec<SpudData>,
    filename_stem: &str,
    csv_output_dir: &str,
    report_date: NaiveDate,
) -> Result<(), AppError> {
    if spud_data.is_empty() {
        return Ok(());
    }

    let output_filename = format!("{}_{}.csv", filename_stem, report_date.format("%Y%m%d"));

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b',')
        .from_path(format!("{csv_output_dir}/{output_filename}"))?;
    for data in spud_data {
        wtr.serialize(data)?;
    }
    wtr.flush()?;
    Ok(())
}

fn extract_date(lines: &[String]) -> Result<NaiveDate, AppError> {
    let date_line = lines
        .get(1)
        .ok_or_else(|| AppError::FileProcessing("Date line not found".to_string()))?;
    let date_str = date_line
        .split_whitespace()
        .skip(2)
        .take(3)
        .collect::<Vec<&str>>()
        .join(" ");
    let parsed_date = NaiveDate::parse_from_str(&date_str, "%d %B %Y")?;
    Ok(parsed_date)
}

pub async fn process_file(
    filename_stem: &str,
    txt_input_dir: &str,
    csv_output_dir: &str,
) -> Result<NaiveDate, AppError> {
    let lines = open_file_lines(&format!("{txt_input_dir}/{filename_stem}.TXT"))?;
    let lines_trimmed = trim_and_remove_empty_lines(lines);

    let extracted_date = extract_date(&lines_trimmed)?;

    let (spud_data_lines, separator_line) = extract_data_and_separator(&lines_trimmed);
    if let Some(separator) = separator_line {
        let spud_data = extract_spud_data(spud_data_lines, extracted_date, &separator);
        if !spud_data.is_empty() {
            write_spud_data_to_csv(spud_data, filename_stem, csv_output_dir, extracted_date)?;
        }
    } else {
        return Err(AppError::FileProcessing(
            "Separator line not found in file".to_string(),
        ));
    }

    Ok(extracted_date)
}

use std::path::PathBuf;

pub async fn process_folder(folder_path: &str, csv_output_dir: &str) -> Result<(), AppError> {
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
            let _ = process_file(&filename_stem, &folder_path_clone, &csv_output_dir).await; // Year is not used in process_folder
            Ok(())
        })
    })
    .await
}
