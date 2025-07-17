use aer_st1::{process_date_range, process_file, process_folder, AppError, ReportType};
use chrono::NaiveDate;
use log::{error, info};
use std::env;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        error!("Usage: {} <report_type> <file|folder|date_range> [args...]", args[0]);
        error!("Example: {} st1 file TXT/WELLS0101.TXT", args[0]);
        error!("Example: {} st49 date_range 2025-01-01 2025-01-31", args[0]);
        std::process::exit(1);
    }

    let report_type: ReportType = args[1].parse()?;
    let command = &args[2];

    match command.as_str() {
        "file" => {
            if args.len() != 4 {
                return Err(AppError::Cli(format!("Usage: {} {} file <filename>", args[0], args[1])));
            }
            let filename = &args[3];
            info!("Processing file: {}", filename);
            process_file(report_type, filename).await?;
        }
        "folder" => {
            if args.len() != 4 {
                return Err(AppError::Cli(format!("Usage: {} {} folder <folder_path>", args[0], args[1])));
            }
            let folder_path = &args[3];
            info!("Processing folder: {}", folder_path);
            process_folder(report_type, folder_path).await?;
        }
        "date_range" => {
            if args.len() != 5 {
                return Err(AppError::Cli(format!("Usage: {} {} date_range <start_date> <end_date>", args[0], args[1])));
            }
            let start_date = NaiveDate::parse_from_str(&args[3], "%Y-%m-%d")?;
            let end_date = NaiveDate::parse_from_str(&args[4], "%Y-%m-%d")?;
            info!("Downloading and processing from {} to {}", start_date, end_date);
            process_date_range(report_type, start_date, end_date).await?;
        }
        _ => {
            return Err(AppError::Cli(format!("Invalid command: {}. Use 'file', 'folder', or 'date_range'.", command)));
        }
    }

    info!("Processing complete.");
    Ok(())
}
