use aer_st1::{
    process_date_range, process_file, process_folder, process_zip_folder, AppError, ReportType,
};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use log::info;
mod delta;

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
        #[arg(long, value_enum)]
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
        #[arg(long, value_enum)]
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
        #[arg(long, value_enum)]
        report_type: ReportType,
        /// The start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: NaiveDate,
        /// The end date (YYYY-MM-DD)
        #[arg(long)]
        end_date: NaiveDate,
        /// Optional: Output directory for TXT files
        #[arg(long, default_value = "TXT")]
        txt_output_dir: String,
        /// Optional: Output directory for CSV files
        #[arg(long, default_value = "CSV")]
        csv_output_dir: String,
    },
    /// Process all files in a zip folder
    Zip {
        /// The type of report to process (st1 or st49)
        #[arg(long, value_enum)]
        report_type: ReportType,
        /// The path to the folder to process
        folder_path: String,
        /// Optional: Output directory for CSV files
        #[arg(long, default_value = "CSV")]
        csv_output_dir: String,
    },
    /// Load CSV(s) into a Delta table
    LoadDelta {
        /// The type of report to load (st1 or st49)
        #[arg(long, value_enum)]
        report_type: ReportType,
        /// Path to a single CSV file (optional if csv_folder is used)
        #[arg(long)]
        csv_path: Option<String>,
        /// Path to a folder containing CSV files (optional if csv_path is used)
        #[arg(long)]
        csv_folder: Option<String>,
        /// Path to the Delta table
        #[arg(long)]
        table_path: String,
        /// Optional: Path to the log file. Defaults to delta_load_log.json inside the table_path.
        #[arg(long)]
        log_path: Option<String>,
        /// Recreate the table if it already exists
        #[arg(long)]
        recreate_table: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::File {
            report_type,
            filename,
            csv_output_dir,
        } => {
            info!("Processing file: {}", filename);
            process_file(*report_type, filename, csv_output_dir, None, None).await?;
        }
        Commands::Folder {
            report_type,
            folder_path,
            csv_output_dir,
        } => {
            info!("Processing folder: {}", folder_path);
            process_folder(*report_type, folder_path, csv_output_dir).await?;
        }
        Commands::DateRange {
            report_type,
            start_date,
            end_date,
            txt_output_dir,
            csv_output_dir,
        } => {
            info!(
                "Downloading and processing from {} to {}",
                start_date, end_date
            );
            process_date_range(
                *report_type,
                *start_date,
                *end_date,
                txt_output_dir,
                csv_output_dir,
            )
            .await?;
        }
        Commands::Zip {
            report_type,
            folder_path,
            csv_output_dir,
        } => {
            info!("Processing zip folder: {}", folder_path);
            process_zip_folder(*report_type, folder_path, csv_output_dir).await?;
        }
        Commands::LoadDelta {
            report_type,
            csv_path,
            csv_folder,
            table_path,
            log_path,
            recreate_table,
        } => {
            use crate::delta::{
                create_or_open_delta_table, load_csv_to_delta, log_loaded_csv, read_load_log,
                DeltaReportType,
            };
            use deltalake::DeltaOps;
            use std::fs;
            use std::path::Path;

            use std::path::PathBuf;

            let log_path = if let Some(lp) = log_path {
                PathBuf::from(lp)
            } else {
                PathBuf::from(table_path).join("delta_load_log.json")
            };

            if *recreate_table {
                let table_path_obj = Path::new(table_path);
                if table_path_obj.exists() {
                    info!("Recreating delta table at {}", table_path);
                    std::fs::remove_dir_all(table_path_obj)?;
                }
                if log_path.exists() {
                    info!("Removing log file at {:?}", log_path);
                    std::fs::remove_file(&log_path)?;
                }
            }

            let delta_type = match report_type {
                ReportType::St1 => DeltaReportType::St1,
                ReportType::St49 => DeltaReportType::St49,
            };

            let mut table = create_or_open_delta_table(Path::new(table_path), delta_type).await?;

            let processed_files = read_load_log(&log_path)?;

            let mut csv_files = Vec::new();
            if let Some(folder) = csv_folder {
                info!("Searching for CSV files in folder: {}", folder);
                let prefix = match report_type {
                    ReportType::St1 => "WELLS",
                    ReportType::St49 => "SPUD",
                };

                for entry in fs::read_dir(folder)? {
                    let entry = entry?;
                    let path = entry.path();
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if filename.starts_with(prefix) && filename.ends_with(".csv") {
                            let canonical_path = path.canonicalize()?;
                            if !processed_files
                                .contains(&canonical_path.to_string_lossy().to_string())
                            {
                                info!("Found CSV file: {:?}", path);
                                csv_files.push(path);
                            } else {
                                info!("Skipping already processed file: {:?}", path);
                            }
                        }
                    }
                }
            }
            if let Some(file) = csv_path {
                let path = Path::new(file).to_path_buf();
                let canonical_path = path.canonicalize()?;
                if !processed_files.contains(&canonical_path.to_string_lossy().to_string()) {
                    csv_files.push(path);
                } else {
                    info!("Skipping already processed file: {:?}", path);
                }
            }

            for csv in csv_files {
                match load_csv_to_delta(&mut table, &csv).await {
                    Ok(loaded_rows) => {
                        info!("Loaded {} rows from {:?}", loaded_rows, csv);
                        log_loaded_csv(&log_path, &csv)?;
                    }
                    Err(e) => {
                        eprintln!("Failed to load CSV file {:?}: {}", csv, e);
                    }
                }
            }

            info!("Optimizing delta table at {}", table_path);
            let ops = DeltaOps::from(table.clone());
            ops.optimize().await?;
            info!("Vacuuming delta table at {}", table_path);
            let ops = DeltaOps::from(table.clone());
            ops.vacuum().with_dry_run(false).await?;
        }
    }

    info!("Processing complete.");
    Ok(())
}
