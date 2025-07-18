use crate::AppError;
use log::{info, warn};
use std::fs::{self, File};
use std::future::Future;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn open_file_lines(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let content = String::from_utf8_lossy(&buffer).into_owned();
    Ok(content.lines().map(|s| s.to_string()).collect())
}

pub async fn process_folder_generic<F, Fut>(
    folder_path: &str,
    file_filter: &str,
    file_processor: F,
) -> Result<(), AppError>
where
    F: Fn(&str) -> Fut,
    Fut: Future<Output = Result<(), AppError>>,
{
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
                if filename.contains(file_filter) && filename.ends_with(".TXT") {
                    info!("Processing file: {}", filename);
                    if let Err(e) = file_processor(filename).await {
                        warn!("Failed to process file {}: {}", filename, e);
                    }
                }
            }
        }
    }
    Ok(())
}
