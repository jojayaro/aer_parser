use crate::error::AppError;
use crate::ReportType;
use chrono::{Duration, NaiveDate};
use futures::future::join_all;
use log::{debug, error, info};
use reqwest;
use std::fs::File;
use std::io::Write;

async fn download_and_save_file(client: &reqwest::Client, url: &str, file_path: &str) -> Result<(), AppError> {
    info!("Downloading file from {}", url);
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(AppError::Download(format!(
            "Failed to download file from {}: status {}",
            url,
            response.status()
        )));
    }

    let bytes = response.bytes().await?;
    debug!("Saving file to {}", file_path);
    let mut file = File::create(file_path)?;
    file.write_all(&bytes)?;
    info!("Successfully downloaded and saved file to {}", file_path);
    Ok(())
}

async fn download_file(
    client: &reqwest::Client,
    date: NaiveDate,
    report_type: &ReportType,
) -> Result<String, AppError> {
    let (url_prefix, file_prefix, extension) = match report_type {
        ReportType::St1 => (
            "https://static.aer.ca/prd/data/well-lic/WELLS",
            "WELLS",
            "TXT",
        ),
        ReportType::St49 => ("https://static.aer.ca/prd/data/wells/SPUD", "SPUD", "txt"),
    };

    let filename_date = date.format("%m%d").to_string();
    let url = format!("{}{}.{}", url_prefix, filename_date, extension);
    let filepath = format!("TXT/{}{}.{}", file_prefix, filename_date, extension);

    download_and_save_file(client, &url, &filepath).await?;

    Ok(filename_date)
}

pub async fn download_files_by_date_range(
    report_type: ReportType,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<String>, AppError> {
    let client = reqwest::Client::new();
    let mut tasks = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        tasks.push(download_file(&client, current_date, &report_type));
        current_date += Duration::days(1);
    }

    let results = join_all(tasks).await;
    let mut downloaded_files = Vec::new();
    for result in results {
        match result {
            Ok(filename) => downloaded_files.push(filename),
            Err(e) => error!("A file download failed: {}", e), // Log error but continue
        }
    }

    Ok(downloaded_files)
}
