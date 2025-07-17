pub mod error;
pub mod downloader;
pub mod st1;
pub mod st49;

use chrono::NaiveDate;
pub use error::AppError;

#[derive(Clone, Copy)]
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

pub async fn process_file(report_type: ReportType, filename: &str) -> Result<(), AppError> {
    match report_type {
        ReportType::St1 => st1::process_file(filename).await,
        ReportType::St49 => st49::process_file(filename).await,
    }
}

pub async fn process_folder(report_type: ReportType, folder_path: &str) -> Result<(), AppError> {
    match report_type {
        ReportType::St1 => st1::process_folder(folder_path).await,
        ReportType::St49 => st49::process_folder(folder_path).await,
    }
}

pub async fn process_date_range(
    report_type: ReportType,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(), AppError> {
    let downloaded_files =
        downloader::download_files_by_date_range(report_type, start_date, end_date).await?;
    
    let report_prefix = match report_type {
        ReportType::St1 => "WELLS",
        ReportType::St49 => "SPUD",
    };

    for filename in downloaded_files {
        let full_filename = format!("TXT/{}{}.TXT", report_prefix, filename);
        match report_type {
            ReportType::St1 => st1::process_file(&full_filename).await?,
            ReportType::St49 => st49::process_file(&full_filename).await?,
        }
    }
    Ok(())
}
