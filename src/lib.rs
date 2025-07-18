pub mod downloader;
pub mod error;
pub mod st1;
pub mod st49;
pub mod utils;

use chrono::NaiveDate;
use clap::ValueEnum;
pub use error::AppError;
use futures::stream::{self, StreamExt};
use std::fs;
use std::io;
use std::path::Path;
use tempfile::NamedTempFile;
use zip::ZipArchive;
use log::info;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReportType {
    St1,
    St49,
}

impl std::str::FromStr for ReportType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "st1" => Ok(ReportType::St1),
            "st49" => Ok(ReportType::St49),
            _ => Err(AppError::Cli(format!("Invalid report type: {s}"))),
        }
    }
}

pub async fn process_file(
    report_type: ReportType,
    filename: &str,
    csv_output_dir: &str,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
) -> Result<(), AppError> {
    let processed_date = match report_type {
        ReportType::St1 => st1::process_file(filename, csv_output_dir).await?,
        ReportType::St49 => st49::process_file(filename, csv_output_dir).await?,
    };

    if let (Some(s_date), Some(e_date)) = (start_date, end_date) {
        if processed_date < s_date || processed_date > e_date {
            return Err(AppError::FileProcessing(format!(
                "Date in file content ({processed_date}) is outside the specified range ({s_date} - {e_date}) for file: {filename}"
            )));
        }
    }
    Ok(())
}

pub async fn move_to_conversion_errors(
    file_path: &Path,
    error_message: &str,
) -> Result<(), AppError> {
    let conversion_errors_dir = Path::new("conversion_errors");
    if !conversion_errors_dir.exists() {
        fs::create_dir_all(conversion_errors_dir)?;
    }
    let new_path = conversion_errors_dir.join(file_path.file_name().unwrap());
    fs::rename(file_path, &new_path)?;
    eprintln!(
        "Failed to process file {file_path:?}: {error_message}. Moved to {new_path:?}"
    );
    Ok(())
}

pub async fn process_folder(
    report_type: ReportType,
    folder_path: &str,
    csv_output_dir: &str,
) -> Result<(), AppError> {
    let entries = fs::read_dir(folder_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let futures = stream::iter(entries)
        .map(|path| {
            let csv_output_dir_clone = csv_output_dir.to_string();
            async move {
                if path.is_file() {
                    if let Some(filename) = path.to_str() {
                        if let Err(e) = process_file(
                            report_type,
                            filename,
                            &csv_output_dir_clone,
                            None,
                            None,
                        )
                        .await
                        {
                            eprintln!("Failed to process file {filename:?}: {e}");
                        }
                    }
                }
            }
        })
        .buffer_unordered(10) // 10 concurrent tasks
        .collect::<()>();

    futures.await;

    Ok(())
}

pub async fn process_date_range(
    report_type: ReportType,
    start_date: NaiveDate,
    end_date: NaiveDate,
    txt_output_dir: &str,
    csv_output_dir: &str,
) -> Result<(), AppError> {
    let downloaded_files =
        downloader::download_files_by_date_range(report_type, start_date, end_date, txt_output_dir)
            .await?;

    let futures = stream::iter(downloaded_files)
        .map(|filename| {
            let txt_output_dir_clone = txt_output_dir.to_string();
            let csv_output_dir_clone = csv_output_dir.to_string();
            async move {
                let (prefix, extension) = match report_type {
                    ReportType::St1 => ("WELLS", "TXT"),
                    ReportType::St49 => ("SPUD", "txt"),
                };
                let full_filename = format!(
                    "{txt_output_dir_clone}/{prefix}{filename}.{extension}"
                );

                // Date validation
                let contents = fs::read_to_string(&full_filename)?;
                let date_str = match report_type {
                    ReportType::St1 => contents.lines().nth(6).unwrap_or_default(),
                    ReportType::St49 => contents.lines().nth(1).unwrap_or_default(),
                };
                let date = match report_type {
                    ReportType::St1 => NaiveDate::parse_from_str(&date_str.trim().replace("DATE: ", ""), "%d %B %Y").unwrap_or_default(),
                    ReportType::St49 => NaiveDate::parse_from_str(date_str.split_whitespace().skip(2).take(3).collect::<Vec<&str>>().join(" ").as_str(), "%d %B %Y").unwrap_or_default(),
                };

                if date >= start_date && date <= end_date {
                    info!("Processing file {full_filename} with date {date} within range");
                    if let Err(e) = process_file(
                        report_type,
                        &full_filename,
                        &csv_output_dir_clone,
                        Some(start_date),
                        Some(end_date),
                    )
                    .await
                    {
                        eprintln!("Failed to process file {full_filename:?}: {e}");
                        move_to_conversion_errors(Path::new(&full_filename), &e.to_string()).await?;
                    }
                } else {
                    println!(
                        "Skipping file {full_filename} with date {date} outside of range"
                    );
                    move_to_conversion_errors(Path::new(&full_filename), "Date outside of range").await?;
                }
                Ok::<(), AppError>(())
            }
        })
        .buffer_unordered(10) // 10 concurrent tasks
        .collect::<Vec<_>>();

    for future in futures.await {
        if let Err(e) = future {
            eprintln!("An error occurred: {e}");
        }
    }

    Ok(())
}

pub async fn process_single_zip_file(
    zip_file_path: &Path,
    report_type: ReportType,
    csv_output_dir: &str,
) -> Result<(), AppError> {
    let file = fs::File::open(zip_file_path)?;
    let mut archive = ZipArchive::new(file)?;
    let year = zip_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.chars().take(4).collect::<String>().parse::<u32>().ok())
        .ok_or_else(|| AppError::FileProcessing(format!("Could not extract year from zip filename: {zip_file_path:?}")))?;

    let txt_output_dir = Path::new("TXT");
    if !txt_output_dir.exists() {
        fs::create_dir_all(txt_output_dir)?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if file.is_dir() {
            continue;
        }

        let new_filename = format!(
            "{_file_stem}{_year}.{_extension}",
            _file_stem = outpath.file_stem().unwrap().to_str().unwrap(),
            _year = year,
            _extension = outpath.extension().unwrap().to_str().unwrap()
        );
        let extracted_file_path = txt_output_dir.join(&new_filename);

        if outpath.extension().and_then(|s| s.to_str()) == Some("zip") {
            // Nested zip file
            let mut temp_zip_file = NamedTempFile::new()?;
            io::copy(&mut file, &mut temp_zip_file)?;
            Box::pin(process_single_zip_file(temp_zip_file.path(), report_type, csv_output_dir)).await?;
            temp_zip_file.close()?;
        } else if outpath.extension().and_then(|s| s.to_str()) == Some("TXT") || outpath.extension().and_then(|s| s.to_str()) == Some("txt") {
            // Report file
            let mut outfile = fs::File::create(&extracted_file_path)?;
            io::copy(&mut file, &mut outfile)?;
            info!("Extracted file to: {extracted_file_path:?}");

            if let Err(e) = process_file(
                report_type,
                extracted_file_path.to_str().unwrap(),
                csv_output_dir,
                None,
                None,
            )
            .await
            {
                move_to_conversion_errors(&extracted_file_path, &e.to_string()).await?;
            }
        } else {
            info!("Skipping unknown file type: {outpath:?}");
        }
    }
    Ok(())
}

pub async fn process_zip_folder(
    report_type: ReportType,
    folder_path: &str,
    csv_output_dir: &str,
) -> Result<(), AppError> {
    let entries = fs::read_dir(folder_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    for path in entries {
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("zip") {
            process_single_zip_file(&path, report_type, csv_output_dir).await?;
        }
    }

    Ok(())
}
