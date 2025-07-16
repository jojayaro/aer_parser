use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufRead, Write};
use chrono::{NaiveDate, Duration};
use reqwest;
use serde::{Serialize, Deserialize};
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

async fn download_file(filename: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://static.aer.ca/prd/data/well-lic/WELLS{}.TXT", filename);

    let response = reqwest::get(&url).await?;

    let bytes = response.bytes().await?;

    let mut file = File::create(format!("TXT/WELLS{}.TXT", filename))?;

    file.write_all(&bytes)?;

    Ok(())
}

async fn download_files_by_date_range(start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut downloaded_files = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        let filename = current_date.format("%m%d").to_string();
        download_file(filename.clone()).await?;
        downloaded_files.push(filename);
        current_date += Duration::days(1);
    }

    Ok(downloaded_files)
}

fn open_file_lines(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let content = BufReader::new(file);
    let lines: Vec<String> = content
        .lines()
        .collect::<Result<_, _>>()?;
    Ok(lines)
}

fn trim_and_remove_empty_lines(lines: Vec<String>) -> Vec<String> {
    let mut lines_trimmed: Vec<String> = Vec::new();

    for line in lines {
        if line.trim().len() > 0 {
            lines_trimmed.push(line.trim().to_string());
        }
    }

    lines_trimmed
}

fn extract_licences_lines(lines: &Vec<String>, breaks: Vec<usize>) -> Vec<String> {
    let mut licences_lines: Vec<String> = Vec::new();

    if breaks.len() < 2 {
        return licences_lines;
    }

    let end_index = if breaks.len() > 2 {
        breaks[2]
    } else {
        lines.len()
    };

    for i in breaks[1] + 1..end_index {
        if lines[i].contains("AMENDMENTS OF WELL LICENCES") || lines[i].contains("END OF WELL LICENCES DAILY LIST") {
            break;
        }
        licences_lines.push(lines[i].to_string());
    }

    licences_lines
}

fn extract_license(lines: Vec<String>, date: String) -> Vec<License> {
    let mut licences: Vec<License> = Vec::new();

    for i in (0..lines.len()).step_by(5) {
        if i + 4 < lines.len() {
            let line0 = &lines[i];
            let line1 = &lines[i+1];
            let line2 = &lines[i+2];
            let line3 = &lines[i+3];
            let line4 = &lines[i+4];

            licences.push(License {
                date: date.clone(),
                well_name: line0.get(..37).unwrap_or("").trim().to_string(),
                licence_number: line0.get(37..47).unwrap_or("").trim().to_string(),
                mineral_rights: line0.get(47..68).unwrap_or("").trim().to_string(),
                ground_elevation: line0.get(68..).unwrap_or("").trim().to_string(),
                unique_identifier: line1.get(..37).unwrap_or("").trim().to_string(),
                surface_coordinates: line1.get(37..47).unwrap_or("").trim().to_string(),
                aer_field_centre: line1.get(47..68).unwrap_or("").trim().to_string(),
                projected_depth: line1.get(68..).unwrap_or("").trim().to_string(),
                aer_classification: line2.get(..37).unwrap_or("").trim().to_string(),
                field: line2.get(37..68).unwrap_or("").trim().to_string(),
                terminating_zone: line2.get(68..).unwrap_or("").trim().to_string(),
                drilling_operation: line3.get(..37).unwrap_or("").trim().to_string(),
                well_purpose: line3.get(37..47).unwrap_or("").trim().to_string(),
                well_type: line3.get(47..68).unwrap_or("").trim().to_string(),
                substance: line3.get(68..).unwrap_or("").trim().to_string(),
                licensee: line4.get(..68).unwrap_or("").trim().to_string(),
                surface_location: line4.get(68..).unwrap_or("").trim().to_string()
            });
        }
    }

    licences
}

fn write_licence_to_csv(licences: Vec<License>, filename: &str) {
    let output_filename = if filename.starts_with("WELLS") && filename.ends_with(".TXT") {
        &filename[..filename.len() - 4]
    } else if !filename.starts_with("WELLS") {
        &format!("WELLS{}", filename)[..filename.len() + 5]
    } else {
        filename
    };
    
    let mut wtr = csv::Writer::from_path(format!("CSV/{}.csv", output_filename))
        .expect("Unable to create csv file");

    for licence in licences {
        wtr.serialize(licence).expect("Unable to write licence to csv file");
    }

    wtr.flush().expect("Unable to flush csv file");
}

#[derive(Debug)]
struct Indices {
    breaks: Vec<usize>,
    date: Vec<usize>,
}

impl Indices {
    fn search(lines: &Vec<String>) -> Indices {
        let mut index_breaks: Vec<usize> = Vec::new();
        let mut index_date: Vec<usize> = Vec::new();

        for (pos, line) in lines.iter().enumerate() {
            if line.contains("---") && !line.contains("END OF") {
                index_breaks.push(pos);
            } else if line.contains("DATE") {
                index_date.push(pos);
            }
        }

        Indices {
            breaks: index_breaks,
            date: index_date,
        }
    }
}

fn process_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path_buf;
    let path_to_open = if filename.contains('/') {
        filename
    } else {
        path_buf = format!("TXT/WELLS{}.TXT", filename);
        &path_buf
    };

    let lines = open_file_lines(path_to_open)?;
    let lines_trimmed = trim_and_remove_empty_lines(lines);
    let index = Indices::search(&lines_trimmed);
    
    if index.date.is_empty() {
        return Err(format!("No date found in file: {}", filename).into());
    }

    let date_str = &lines_trimmed[index.date[0]].trim()[6..];
    let parsed_date = NaiveDate::parse_from_str(date_str, "%d %B %Y")?;
    let formatted_date = parsed_date.format("%Y-%m-%d").to_string();

    let licences_lines = extract_licences_lines(&lines_trimmed, index.breaks);
    let licences = extract_license(licences_lines, formatted_date);
    let filename_for_csv = Path::new(filename).file_name().and_then(|s| s.to_str()).unwrap_or(filename);
    write_licence_to_csv(licences, filename_for_csv);
    Ok(())
}

fn process_folder(folder_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(folder_path);
    if !folder.is_dir() {
        return Err(format!("{} is not a valid directory", folder_path).into());
    }

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("TXT") {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with("WELLS") {
                    println!("Processing file: {}", path.display());
                    process_file(path.to_str().unwrap_or_default())?;
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file|folder|date_range> [args...]", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "file" => {
            if args.len() != 3 {
                eprintln!("Usage: {} file <filename>", args[0]);
                std::process::exit(1);
            }
            let filename = &args[2];
            process_file(filename)?;
        }
        "folder" => {
            if args.len() != 3 {
                eprintln!("Usage: {} folder <folder_path>", args[0]);
                std::process::exit(1);
            }
            let folder_path = &args[2];
            process_folder(folder_path)?;
        }
        "date_range" => {
            if args.len() != 4 {
                eprintln!("Usage: {} date_range <start_date> <end_date>", args[0]);
                std::process::exit(1);
            }
            let start_date = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d")?;
            let end_date = NaiveDate::parse_from_str(&args[3], "%Y-%m-%d")?;
            let downloaded_files = download_files_by_date_range(start_date, end_date).await?;
            for filename in downloaded_files {
                process_file(&filename)?;
            }
        }
        _ => {
            eprintln!("Invalid option. Use 'file', 'folder', or 'date_range'.");
            std::process::exit(1);
        }
    }

    Ok(())
}
