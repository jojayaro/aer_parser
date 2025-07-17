pub mod error;
pub mod downloader;
pub mod st1;
pub mod st49;
pub mod utils;

use chrono::NaiveDate;
pub use error::AppError;
use clap::ValueEnum;

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
            _ => Err(AppError::Cli(format!("Invalid report type: {}", s))),
        }
    }
}

pub async fn process_file(report_type: ReportType, filename: &str, csv_output_dir: &str) -> Result<(), AppError> {
    match report_type {
        ReportType::St1 => st1::process_file(filename, csv_output_dir).await,
        ReportType::St49 => st49::process_file(filename, csv_output_dir).await,
    }
}

pub async fn process_folder(report_type: ReportType, folder_path: &str, csv_output_dir: &str) -> Result<(), AppError> {
    match report_type {
        ReportType::St1 => st1::process_folder(folder_path, csv_output_dir).await,
        ReportType::St49 => st49::process_folder(folder_path, csv_output_dir).await,
    }
}

pub async fn process_date_range(
    report_type: ReportType,
    start_date: NaiveDate,
    end_date: NaiveDate,
    txt_output_dir: &str,
    csv_output_dir: &str,
) -> Result<(), AppError> {
    let downloaded_files =
        downloader::download_files_by_date_range(report_type, start_date, end_date, txt_output_dir).await?;
    
    for filename in downloaded_files {
        let (prefix, extension) = match report_type {
            ReportType::St1 => ("WELLS", "TXT"),
            ReportType::St49 => ("SPUD", "txt"),
        };
        let full_filename = format!("{}/{}{}.{}", txt_output_dir, prefix, filename, extension);
        if let Err(e) = process_file(report_type, &full_filename, csv_output_dir).await {
            eprintln!("Failed to process file {:?}: {}", full_filename, e);
        }
    }
    Ok(())
}
