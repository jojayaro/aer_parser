# CLI Rules

This document outlines the usage of the `aer_parser` command-line interface.

## General Principles

*   **Small, Incremental Changes:** I will make small, focused changes to the codebase.
*   **Document the codebase:** I will document the codebase thoroughly, including comments and documentation for public functions and modules.
*   **Test-Driven Development (TDD):** I will write tests before implementing new features or fixing bugs.
*   **Run Tests Frequently:** I will run the test suite frequently to ensure that the codebase remains stable.
*   **Use of `cargo clippy`:** I will run `cargo clippy` to check for common mistakes and improve code quality.
*   **Use of `cargo fmt`:** I will run `cargo fmt` to ensure that the code is formatted consistently.
*   **Test After Each Change:** After every change, I will run the appropriate tests to ensure that the change has not introduced any regressions.
*   **Commit Frequently:** I will commit my changes frequently, with clear and descriptive commit messages.
*   **Update Readme.md:** I will update the `README.md` file with any new features or changes to the CLI commands.
*   **Follow Best Practices:** I will adhere to Rust best practices for code style, error handling, and performance.
*   **Clear Communication:** I will keep you informed of my progress and any issues I encounter.
*   **Use of `clap` for CLI arguments:** I will use the `clap` crate to handle command-line arguments and options, ensuring a user-friendly interface.
*   **Stop when in doubt and ask questions:** If I am unsure about a specific implementation detail or design decision, I will stop and ask for clarification.
*   **Prioritize simplicity and maintainability:** I will strive to keep the code simple and easy to maintain, avoiding unnecessary complexity. No unsafe code is allowed in the project or dependencies.
*   **If I can't find a reference file I will check .gitignore first before asking for help.** This will help avoid unnecessary questions and speed up the development process.

## Error Handling and Logging

*   **Use `thiserror` for custom error types:** I will use the `thiserror` crate to create custom error types that provide clear and informative error messages.
*   **Implement comprehensive error handling:** I will ensure that all potential errors are handled gracefully.
*   **Add logging where appropriate:** I will add logging statements to help with debugging and monitoring the application.

## Refactoring

*   **Break down large functions:** I will break down large functions into smaller, more manageable functions.
*   **Improve readability:** I will improve the readability of the code by using clear and concise variable names, and by adding comments where necessary.

## Commands

### `file`

Processes a single raw report file and converts it to a CSV.

**Arguments:**
-   `--report-type`: The type of report to process (`st1` or `st49`).
-   `filename`: The path to the file to process.
-   `--csv-output-dir`: (Optional) The directory to save the generated CSV file. Defaults to `data/csv`.

**Usage:**
```bash
aer_parser file --report-type st1 /path/to/your/file.txt
```

### `folder`

Processes all raw report files in a given directory.

**Arguments:**
-   `--report-type`: The type of report to process (`st1` or `st49`).
-   `folder_path`: The path to the folder to process.
-   `--csv-output-dir`: (Optional) The directory to save the generated CSV files. Defaults to `data/csv`.

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
-   `--txt-output-dir`: (Optional) The directory to save the downloaded raw files. Defaults to `data/txt`.
-   `--csv-output-dir`: (Optional) The directory to save the generated CSV files. Defaults to `data/csv`.

**Usage:**
```bash
aer_parser date-range --report-type st1 --start-date 2024-01-01 --end-date 2024-01-31
```

### `zip`

Processes all raw report files in a zip folder.

**Arguments:**
-   `--report-type`: The type of report to process (`st1` or `st49`).
-   `folder_path`: The path to the folder to process.
-   `--txt-output-dir`: (Optional) The directory to save the extracted raw files. Defaults to `data/txt`.
-   `--csv-output-dir`: (Optional) The directory to save the generated CSV files. Defaults to `data/csv`.

**Usage:**
```bash
aer_parser zip --report-type st1 /path/to/your/zip_folder
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
duckdb < ./queries/read_delta_st1_csv.sql
```

**ST49 CSV Verification:**
```bash
# Query the st49 CSVs from the CSV/ directory
duckdb < ./queries/read_delta_st49_csv.sql
```

### Delta Table Verification

To verify that the data has been loaded correctly into the Delta table, you can use the following workflow.

**ST1 Verification:**
```bash
# Recreate the table and load all st1 CSVs from the CSV/ directory
RUST_LOG=info cargo run --bin aer_parser load-delta --report-type st1 --csv-folder CSV --table-path st1 --recreate-table

# Query the delta table to verify the contents
duckdb < ./queries/read_delta_st1.sql

# Compare the delta table with the CSV files
duckdb < ./queries/st1_csv_delta_comparison.sql
```

**ST49 Verification:**
```bash
# Recreate the table and load all st49 CSVs from the CSV/ directory
RUST_LOG=info cargo run --bin aer_parser load-delta --report-type st49 --csv-folder CSV --table-path st49 --recreate-table

# Query the delta table to verify the contents
duckdb < ./queries/read_delta_st49.sql

# Compare the delta table with the CSV files
duckdb < ./queries/st49_csv_delta_comparison.sql
```

### Debugging

To see detailed logging output, set the `RUST_LOG` environment variable to `info`. This is useful for seeing which files are being found and loaded.

```bash
RUST_LOG=info cargo run --bin aer_parser ...
