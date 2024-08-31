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
    let path = if filename.starts_with("WELLS") {
        filename.to_string()
    } else {
        format!("TXT/WELLS{}.TXT", filename)
    };
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

    for i in breaks[1]+1..breaks[2]-1 {
        licences_lines.push(lines[i].to_string());
    }

    licences_lines
}

fn extract_license(lines: Vec<String>, date: String) -> Vec<License> {
    let mut licences: Vec<License> = Vec::new();

    let mut index: Vec<usize> = Vec::new();

    for (pos, _) in lines.iter().enumerate() {
        if pos % 5 == 0 {
            index.push(pos);
        }
    }

    for i in index {
        licences.push(License {
            date: date.clone(),
            well_name: lines[i][..37].trim().to_string(),
            licence_number: lines[i][37..47].trim().to_string(),
            mineral_rights: lines[i][47..68].trim().to_string(),
            ground_elevation: lines[i][68..].trim().to_string(),
            unique_identifier: lines[i+1][..37].trim().to_string(),
            surface_coordinates: lines[i+1][37..47].trim().to_string(),
            aer_field_centre: lines[i+1][47..68].trim().to_string(),
            projected_depth: lines[i+1][68..].trim().to_string(),
            aer_classification: lines[i+2][..37].trim().to_string(),
            field: lines[i+2][37..68].trim().to_string(),
            terminating_zone: lines[i+2][68..].trim().to_string(),
            drilling_operation: lines[i+3][..37].trim().to_string(),
            well_purpose: lines[i+3][37..47].trim().to_string(),
            well_type: lines[i+3][47..68].trim().to_string(),
            substance: lines[i+3][68..].trim().to_string(),
            licensee: lines[i+4][..68].trim().to_string(),
            surface_location: lines[i+4][68..].trim().to_string()
        });
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
            if line.contains("---") {
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
    let lines = open_file_lines(filename)?;
    let lines_trimmed = trim_and_remove_empty_lines(lines);
    let index = Indices::search(&lines_trimmed);
    let date = &lines_trimmed[index.date[0]].trim()[6..];
    let licences_lines = extract_licences_lines(&lines_trimmed, index.breaks);
    let licences = extract_license(licences_lines, date.to_string());
    write_licence_to_csv(licences, filename);
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
                    process_file(filename)?;
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
