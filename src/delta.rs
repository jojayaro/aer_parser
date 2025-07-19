use anyhow::{anyhow, Result};
use chrono::Utc;
use delta_kernel::engine::arrow_conversion::TryIntoArrow;
use deltalake::arrow::array::RecordBatch;
use deltalake::arrow::csv::ReaderBuilder;
use deltalake::kernel::{DataType, PrimitiveType, StructField};
use deltalake::protocol::SaveMode;

use deltalake::{DeltaOps, DeltaTable};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use log::warn;

/// Supported report types for delta ingestion.
#[derive(Copy, Clone, Debug)]
pub enum DeltaReportType {
    St1,
    St49,
}

/// Returns the schema for the given report type.
fn get_schema(report_type: DeltaReportType) -> Vec<StructField> {
    let string_field = |name: &str| -> StructField {
        StructField::new(
            name.to_string(),
            DataType::Primitive(PrimitiveType::String),
            true,
        )
    };

    match report_type {
        DeltaReportType::St1 => vec![
            string_field("date"),
            string_field("well_name"),
            string_field("licence_number"),
            string_field("mineral_rights"),
            string_field("ground_elevation"),
            string_field("unique_identifier"),
            string_field("surface_coordinates"),
            string_field("aer_field_centre"),
            string_field("projected_depth"),
            string_field("aer_classification"),
            string_field("field"),
            string_field("terminating_zone"),
            string_field("drilling_operation"),
            string_field("well_purpose"),
            string_field("well_type"),
            string_field("substance"),
            string_field("licensee"),
            string_field("surface_location"),
        ],
        DeltaReportType::St49 => vec![
            string_field("date"),
            string_field("well_id"),
            string_field("well_name"),
            string_field("licence"),
            string_field("contractor_ba_id"),
            string_field("contractor_name"),
            string_field("rig_number"),
            string_field("activity_date"),
            string_field("field_centre"),
            string_field("ba_id"),
            string_field("licensee"),
            string_field("new_projected_total_depth"),
            string_field("activity_type"),
        ],
    }
}

/// Create a delta table at the given path with the appropriate schema.
/// If the table exists, open it.
pub async fn create_or_open_delta_table(
    table_path: &Path,
    report_type: DeltaReportType,
) -> Result<DeltaTable> {
    let table_uri = table_path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid table path"))?;

    if table_path.join("_delta_log").exists() {
        Ok(deltalake::open_table(table_uri).await?)
    } else {
        if let Some(parent) = table_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let ops = DeltaOps::try_from_uri(table_uri).await?;
        let table = ops
            .create()
            .with_save_mode(SaveMode::Ignore)
            .with_columns(get_schema(report_type))
            .await?;
        Ok(table)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct LogEntry {
    csv_file: String,
    timestamp: String,
}

/// Reads the log file and returns a set of processed CSV file paths.
pub fn read_load_log(log_path: &Path) -> Result<HashSet<String>> {
    if !log_path.exists() {
        return Ok(HashSet::new());
    }

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let mut processed_files = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
            processed_files.insert(entry.csv_file);
        }
    }

    Ok(processed_files)
}

/// Logs a successfully loaded CSV file.
pub fn log_loaded_csv(log_path: &Path, csv_path: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let log_entry = LogEntry {
        csv_file: csv_path.canonicalize()?.to_string_lossy().into_owned(),
        timestamp: Utc::now().to_rfc3339(),
    };

    writeln!(file, "{}", serde_json::to_string(&log_entry)?)?;
    Ok(())
}

/// Converts a CSV file to an Arrow RecordBatch.
fn csv_to_record_batch(
    csv_path: &Path,
    schema: Arc<deltalake::arrow::datatypes::Schema>,
) -> Result<RecordBatch> {
    let file = File::open(csv_path)?;
    let csv_reader = ReaderBuilder::new(Arc::clone(&schema))
        .with_header(true)
        .with_delimiter(b'|')
        .build(file)?;

    // Collect all records from the CSV into a single RecordBatch
    let mut batches = Vec::new();
    for batch in csv_reader {
        batches.push(batch?);
    }

    if batches.is_empty() {
        return Err(anyhow!("No data in CSV file: {:?}", csv_path));
    }

    Ok(deltalake::arrow::compute::concat_batches(
        &schema, &batches,
    )?)
}

/// Load multiple CSV files into the delta table as a single batch operation.
/// This creates larger parquet files instead of many small ones.
pub async fn load_csvs_to_delta(
    table: &mut DeltaTable,
    csv_paths: &[&Path],
) -> Result<usize> {
    let arrow_schema = Arc::new(
        table
            .schema()
            .ok_or_else(|| anyhow!("Failed to get table schema"))?
            .try_into_arrow()?,
    );

    let mut all_batches = Vec::new();
    let mut total_rows = 0;

    // Collect all data from all CSV files into a single large batch
    for csv_path in csv_paths {
        match csv_to_record_batch(csv_path, Arc::clone(&arrow_schema)) {
            Ok(batch) => {
                total_rows += batch.num_rows();
                all_batches.push(batch);
            }
            Err(e) => {
                warn!("Could not read CSV file {csv_path:?}, skipping: {e}");
                continue;
            }
        }
    }

    if all_batches.is_empty() {
        return Ok(0);
    }

    // Combine all batches into a single large RecordBatch
    let combined_batch = deltalake::arrow::compute::concat_batches(
        &arrow_schema,
        &all_batches,
    )?;

    if combined_batch.num_rows() > 0 {
        let ops = DeltaOps::from(table.clone());
        
        // Configure write with large target file size (1GB = 1_073_741_824 bytes)
        let table_ref = ops
            .write(vec![combined_batch])
            .with_save_mode(SaveMode::Append)
            .with_target_file_size(1_073_741_824) // 1GB in bytes
            .await?;
        *table = table_ref;
    }

    Ok(total_rows)
}

/// Legacy function for loading a single CSV file (kept for backward compatibility)
pub async fn load_csv_to_delta(table: &mut DeltaTable, csv_path: &Path) -> Result<usize> {
    load_csvs_to_delta(table, &[csv_path]).await
}
