# CLI Rules

This document outlines the usage of the `aer_parser` command-line interface.

## Commands

### `file`

Processes a single raw report file and converts it to a CSV.

**Arguments:**
-   `--report-type`: The type of report to process (`st1` or `st49`).
-   `filename`: The path to the file to process.
-   `--csv-output-dir`: (Optional) The directory to save the generated CSV file. Defaults to `CSV`.

**Usage:**
```bash
aer_parser file --report-type st1 /path/to/your/file.txt
```

### `folder`

Processes all raw report files in a given directory.

**Arguments:**
-   `--report-type`: The type of report to process (`st1` or `st49`).
-   `folder_path`: The path to the folder to process.
-   `--csv-output-dir`: (Optional) The directory to save the generated CSV files. Defaults to `CSV`.

**Usage:**
```bash
aer_parser folder --report-type st49 /path/to/your/folder
```

### `date-range`

Downloads and processes raw report files for a given date range.

**Arguments:**
-   `--report-type`: The type of report to process (`st1` or `st49`).
-   `--start-date`: The start date in `YYYY-MM-DD` format.
-   `--end-date`: The end date in `YYYY-MM-DD` format.
-   `--txt-output-dir`: (Optional) The directory to save the downloaded raw files. Defaults to `TXT`.
-   `--csv-output-dir`: (Optional) The directory to save the generated CSV files. Defaults to `CSV`.

**Usage:**
```bash
aer_parser date-range --report-type st1 --start-date 2024-01-01 --end-date 2024-01-31
```

### `load-delta`

Loads CSV files into a Delta table. This command is idempotent and uses a log file to track which CSVs have already been processed.

**Arguments:**
-   `--report-type`: The type of report to load (`st1` or `st49`).
-   `--csv-folder`: The path to a folder containing CSV files to load.
-   `--table-path`: The path to the Delta table.
-   `--log-path`: (Optional) The path to the log file that tracks processed CSVs. Defaults to `delta_load_log.json`.
-   `--recreate-table`: (Optional) A flag to delete and recreate the Delta table. **This will also delete the log file**, ensuring a fresh load of all CSVs in the source folder.

**Usage:**
```bash
aer_parser load-delta --report-type st1 --csv-folder /path/to/your/csv_folder --table-path /path/to/your/delta_table
```

## Verification and Debugging

### CSV Verification

Before loading into a Delta table, you can verify the contents of the generated CSV files. This is useful for debugging the parsing logic.

**ST1 CSV Verification:**
```bash
# Query the st1 CSVs from the CSV/ directory
duckdb < ./read_delta_st1_csv.sql
```

**ST49 CSV Verification:**
```bash
# Query the st49 CSVs from the CSV/ directory
duckdb < ./read_delta_st49_csv.sql
```

### Delta Table Verification

To verify that the data has been loaded correctly into the Delta table, you can use the following workflow.

**ST1 Verification:**
```bash
# Recreate the table and load all st1 CSVs from the CSV/ directory
RUST_LOG=info cargo run --bin aer_parser load-delta --report-type st1 --csv-folder CSV --table-path st1 --recreate-table

# Query the delta table to verify the contents
duckdb < ./read_delta_st1.sql
```

**ST49 Verification:**
```bash
# Recreate the table and load all st49 CSVs from the CSV/ directory
RUST_LOG=info cargo run --bin aer_parser load-delta --report-type st49 --csv-folder CSV --table-path st49 --recreate-table

# Query the delta table to verify the contents
duckdb < ./read_delta_st49.sql
```

### Debugging

To see detailed logging output, set the `RUST_LOG` environment variable to `info`. This is useful for seeing which files are being found and loaded.

```bash
RUST_LOG=info cargo run --bin aer_parser ...
