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

Loads CSV files into a Delta table.

**Arguments:**
-   `--report-type`: The type of report to load (`st1` or `st49`).
-   `--csv-path`: (Optional) The path to a single CSV file to load.
-   `--csv-folder`: (Optional) The path to a folder containing CSV files to load.
-   `--table-path`: The path to the Delta table.
-   `--log-path`: (Optional) The path to the log file that tracks processed CSVs. Defaults to `delta_load_log.json`.
-   `--recreate-table`: (Optional) A flag to delete and recreate the Delta table before loading.

**Usage:**
```bash
aer_parser load-delta --report-type st1 --csv-folder /path/to/your/csv_folder --table-path /path/to/your/delta_table --recreate-table
```

## Verification

To verify that the data has been loaded correctly, you can use `duckdb` to query the Delta table.

**ST1 Verification:**
```bash
# Load st1 data into delta table
cargo run --bin aer_parser load-delta --report-type st1 --csv-folder /Users/jayaro/Repos/aer_parser/CSV --table-path /Users/jayaro/Repos/aer_parser/st1

# Query the delta table
duckdb < ./read_delta_st1.sql
```

**ST49 Verification:**
```bash
# Load st49 data into delta table
cargo run --bin aer_parser load-delta --report-type st49 --csv-folder /Users/jayaro/Repos/aer_parser/CSV --table-path /Users/jayaro/Repos/aer_parser/st49

# Query the delta table
duckdb < ./read_delta_st49.sql
