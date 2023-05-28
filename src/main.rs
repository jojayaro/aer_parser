use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use reqwest;
use serde::{Serialize, Deserialize};

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

fn open_file_lines(filename: &str) -> Vec<String> {
    let path = format!("TXT/WELLS{}.TXT", filename);
    let file = File::open(path).expect("File not found");
    let content = BufReader::new(file);
    let lines: Vec<String> = content
        .lines()
        .map(|line| line.expect("Something went wrong"))
        .collect();

    lines
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

    for (pos, e) in lines.iter().enumerate() {
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
    let mut wtr = csv::Writer::from_path(format!("CSV/WELLS{}.csv", filename)).expect("Unable to create csv file");

    for licence in licences {
        wtr.serialize(licence).expect("Unable to write licence to csv file");
    }

    wtr.flush().expect("Unable to flush csv file");
}

#[derive(Debug)]
struct Indeces {
    breaks: Vec<usize>,
    date: Vec<usize>,
    cancelled: Vec<usize>,
    amendments: Vec<usize>,
    updates: Vec<usize>,
}

impl Indeces {
    fn search(lines: &Vec<String>) -> Indeces {  
    
        let lines_iter = lines.iter().enumerate();

        let mut index_breaks: Vec<usize> = Vec::new();
        let mut index_date: Vec<usize> = Vec::new();
        let mut index_cancelled: Vec<usize> = Vec::new();
        let mut index_amendments: Vec<usize> = Vec::new();
        let mut index_updates: Vec<usize> = Vec::new();
        
        for (pos, e) in lines_iter {
            if e.contains("---") {
                index_breaks.push(pos);
                //println!("Element at position {}: {:?}", pos, e);
            } else if e.contains("DATE") {
                index_date.push(pos);
                //println!("Element at position {}: {:?}", pos, e);
            } else if e.contains("CANCELLED") {
                index_cancelled.push(pos);
                //println!("Element at position {}: {:?}", pos, e);
            } else if e.contains("AMENDMENTS") {
                index_amendments.push(pos);
                //println!("Element at position {}: {:?}", pos, e);
            } else if e.contains("UPDATES") {
                index_updates.push(pos);
                //println!("Element at position {}: {:?}", pos, e);
            }
        }
        
        let indices = Indeces {
            breaks: index_breaks,
            date: index_date,
            cancelled: index_cancelled,
            amendments: index_amendments,
            updates: index_updates,
        };

        indices
        
    }
}



#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    println!("In file {}", filename);

    //download_file(filename.to_string()).await.unwrap();

    let lines = open_file_lines(filename);

    let lines_trimmed = trim_and_remove_empty_lines(lines);

    let index = Indeces::search(&lines_trimmed);

    let date = &lines_trimmed[index.date[0]].trim()[6..];

    let licences_lines = extract_licences_lines(&lines_trimmed, index.breaks);

    let licences = extract_license(licences_lines, date.to_string());

    write_licence_to_csv(licences, filename);

}
