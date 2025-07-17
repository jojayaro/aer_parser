use chrono::{Duration, NaiveDate};
use futures::future::join_all;
use reqwest;
use std::fs::File;
use std::io::Write;

use crate::{AppError, ReportType};

async fn download_file(
    client: &reqwest::Client,
    date: NaiveDate,
    report_type: &ReportType,
) -> Result<String, AppError> {
    let (url_prefix, file_prefix) = match report_type {
        ReportType::St1 => (
            "https://static.aer.ca/prd/data/well-lic/WELLS",
            "WELLS",
        ),
        ReportType::St49 => ("https://static.aer.ca/prd/data/wells/SPUD", "SPUD"),
    };

    let filename_date = date.format("%m%d").to_string();
    let url = format!("{}{}.TXT", url_prefix, filename_date);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(AppError::Download(format!(
            "Failed to download file from {}: status {}",
            url,
            response.status()
        )));
    }

    let bytes = response.bytes().await?;
    let filepath = format!("TXT/{}{}.TXT", file_prefix, filename_date);
    let mut file = File::create(&filepath)?;
    file.write_all(&bytes)?;

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
            Err(e) => eprintln!("A file download failed: {}", e), // Log error but continue
        }
    }

    Ok(downloaded_files)
}
