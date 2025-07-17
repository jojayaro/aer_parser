use aer_st1::{process_date_range, process_file, process_folder, AppError, ReportType};
use chrono::NaiveDate;
use log::{info};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Process a single file
    File {
        /// The type of report to process (st1 or st49)
        #[arg(value_enum)]
        report_type: ReportType,
        /// The path to the file to process
        filename: String,
        /// Optional: Output directory for CSV files
        #[arg(long, default_value = "CSV")]
        csv_output_dir: String,
    },
    /// Process all files in a folder
    Folder {
        /// The type of report to process (st1 or st49)
        #[arg(value_enum)]
        report_type: ReportType,
        /// The path to the folder to process
        folder_path: String,
        /// Optional: Output directory for CSV files
        #[arg(long, default_value = "CSV")]
        csv_output_dir: String,
    },
    /// Download and process files within a date range
    DateRange {
        /// The type of report to process (st1 or st49)
        #[arg(value_enum)]
        report_type: ReportType,
        /// The start date (YYYY-MM-DD)
        start_date: NaiveDate,
        /// The end date (YYYY-MM-DD)
        end_date: NaiveDate,
        /// Optional: Output directory for TXT files
        #[arg(long, default_value = "TXT")]
        txt_output_dir: String,
        /// Optional: Output directory for CSV files
        #[arg(long, default_value = "CSV")]
        csv_output_dir: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::File { report_type, filename, csv_output_dir } => {
            info!("Processing file: {}", filename);
            process_file(*report_type, filename, csv_output_dir).await?;
        }
        Commands::Folder { report_type, folder_path, csv_output_dir } => {
            info!("Processing folder: {}", folder_path);
            process_folder(*report_type, folder_path, csv_output_dir).await?;
        }
        Commands::DateRange { report_type, start_date, end_date, txt_output_dir, csv_output_dir } => {
            info!("Downloading and processing from {} to {}", start_date, end_date);
            process_date_range(*report_type, *start_date, *end_date, txt_output_dir, csv_output_dir).await?;
        }
    }

    info!("Processing complete.");
    Ok(())
}
