use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead};
use downloader::Downloader;

fn main() {
    //Arguments
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    println!("In file {}", filename);

    //Downloader
    let url = format!("https://static.aer.ca/prd/data/well-lic/{}.TXT", filename);

    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("TXT"))
        .parallel_requests(1)
        .build()
        .unwrap();

    let dl = downloader::Download::new(&url);

    let result = downloader.download(&[dl]).unwrap();

    for r in result {
        match r {
            Err(e) => print!("Error occurred! {}", e.to_string()),
            Ok(s) => print!("Success: {}", &s),
        };
    }

    let lines = open_file_lines(filename);

    let index = Indeces::search(&lines);

    let date = &lines[index.date[0]].trim()[6..];

    if lines.len() > 20 {
        let licences = licences(&lines, &index.breaks);
        writer(licences, &date, 17);
    };
    
    if index.cancelled.len() > 0 {
        let cancelled = cancelled(&lines, &index.breaks, &index.cancelled);
        writer(cancelled, &date, 2);
    }

}

struct Indeces {
    breaks: Vec<usize>,
    date: Vec<usize>,
    cancelled: Vec<usize>,
}

impl Indeces {
    fn search(lines: &Vec<String>) -> Indeces {  
    
        let lines_iter = lines.iter().enumerate();

        let mut index_breaks: Vec<usize> = Vec::new();
        let mut index_date: Vec<usize> = Vec::new();
        let mut index_cancelled: Vec<usize> = Vec::new();
        
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
            }
            };
        
        let indices = Indeces {
            breaks: index_breaks,
            date: index_date,
            cancelled: index_cancelled,
        };

        indices
        
    }
}

fn open_file_lines(filename: &str) -> Vec<String> {
    let path = format!("TXT/{}.TXT", filename);
    let file = File::open(path).expect("File not found");
    let content = BufReader::new(file);
    let lines: Vec<String> = content
    .lines()
    .map(|line| line.expect("Something went wrong"))
    .collect();

    lines
}

fn licences <'a>(lines: &'a Vec<String>, breaks: &'a Vec<usize>) -> Vec<&'a str> {
    let mut licences_vec: Vec<&str> = Vec::new();
    
    for i in breaks[1]+1..breaks[2]-2 {
        if lines[i].trim().len() > 0 {
            licences_vec.push(&lines[i].trim());
        }
    };

    let mut i = 0;
    let mut licence_vec_clean_split: Vec<&str> = Vec::new();

    while i < licences_vec.len() {
        licence_vec_clean_split.push(&licences_vec[i].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i].trim()[37..47]);
        licence_vec_clean_split.push(&licences_vec[i].trim()[47..68]);
        licence_vec_clean_split.push(&licences_vec[i].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+1].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i+1].trim()[37..47]);
        licence_vec_clean_split.push(&licences_vec[i+1].trim()[47..68]);
        licence_vec_clean_split.push(&licences_vec[i+1].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+2].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i+2].trim()[37..68]);
        licence_vec_clean_split.push(&licences_vec[i+2].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+3].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i+3].trim()[37..47]);
        licence_vec_clean_split.push(&licences_vec[i+3].trim()[47..68]);
        licence_vec_clean_split.push(&licences_vec[i+3].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+4].trim()[..68]);
        licence_vec_clean_split.push(&licences_vec[i+4].trim()[68..]);

        i += 5;
    };

    licence_vec_clean_split

}

fn cancelled <'a>(lines: &'a Vec<String>, breaks: &'a Vec<usize>, cancelled: &'a Vec<usize>) -> Vec<&'a str>{
    let mut cancelled_vec_index: Vec<usize> = Vec::new();

    for j in breaks {
        if j > &cancelled[0] {
                    cancelled_vec_index.push(*j);
        }
    }

    let mut cancelled_vec: Vec<&str> = Vec::new();
    
    for i in cancelled[0]+5..cancelled_vec_index[2]-2 {
        if lines[i].trim().len() > 0 {
            cancelled_vec.push(&lines[i].trim());
        }
    };

    let mut c = 0;
    let mut cancelled_vec_clean_split: Vec<&str> = Vec::new();

    while c < cancelled_vec.len() {
        cancelled_vec_clean_split.push(&cancelled_vec[c][..cancelled_vec[c].len()-8].trim());
        cancelled_vec_clean_split.push(&cancelled_vec[c][cancelled_vec[c].len()-7..].trim());
        c += 2;
    }

    cancelled_vec_clean_split

}

fn writer (licences: Vec<&str>, date: &str, x: usize) {
    let number_licences = licences.len()/x;

    let path_lic = format!("ST1_Licences_{}.csv", date);

    //create a csv with the results from licence_vec_clean_split vector divided by 18 columns and write a file
    let csv_file = File::create(path_lic).expect("Unable to create file");
    let mut csv_writer = csv::Writer::from_writer(csv_file);

    for i in 0..number_licences {
        let mut row = Vec::new();
        for j in 0..x {
            row.push(licences[i*x+j].trim());
        }
        row.push(date);

        //println!("{:?}", row);

        csv_writer.write_record(&row).expect("Unable to write to file");
    }

    csv_writer.flush().expect("Unable to flush to file");
}