# ST1 Alberta Energy Regulator Parser

## Overview

This high-performance Rust-based parser efficiently processes the Alberta Energy Regulator's ST1 and ST49 data, which contains critical information on oil and gas exploration licenses and drilling activities. The ST1 and ST49 reports, available from the AER's [official page](https://www.aer.ca/providing-information/data-and-reports/statistical-reports/st1), provide valuable insights into the regulatory landscape of Alberta's energy sector.

## Key Features

- **Asynchronous File Downloading**: Utilizes `tokio` and `reqwest` for efficient, non-blocking file retrieval.
- **Robust Parsing Algorithm**: Implements a sophisticated parsing mechanism to extract structured data from raw text files.
- **CSV Output**: Generates clean, analysis-ready CSV files for seamless integration with data processing pipelines.
- **Delta Lake Integration**: Efficiently loads processed CSV data into Delta Lake tables, with built-in optimization and vacuuming for performance and storage management.
- **Error Handling**: Comprehensive error management for resilient operation.
- **Memory Efficient**: Employs Rust's ownership model for optimal memory usage when processing large datasets.

## Usage

### File Retrieval and Processing

- **Process a single file**: `cargo run file --report-type <st1|st49> <filename> --csv-output-dir <output_directory>`
  Example: `cargo run file --report-type st1 WELLS20230101.TXT --csv-output-dir CSV`

- **Process all files in a folder**: `cargo run folder --report-type <st1|st49> <folder_path> --csv-output-dir <output_directory>`
  Example: `cargo run folder --report-type st49 ./TXT --csv-output-dir CSV`

- **Download and process files for a date range**: `cargo run date-range --report-type <st1|st49> --start-date <YYYY-MM-DD> --end-date <YYYY-MM-DD> --txt-output-dir <txt_output_directory> --csv-output-dir <csv_output_directory>`
  Example: `cargo run date-range --report-type st1 --start-date 2023-01-01 --end-date 2023-01-31 --txt-output-dir TXT --csv-output-dir CSV`

### Loading Data into Delta Lake

After processing files into CSVs, you can load them into a Delta Lake table. This command also performs `OPTIMIZE` and `VACUUM` operations on the Delta table to ensure optimal performance and storage.

- **Load CSV(s) into a Delta table**: `cargo run load-delta --report-type <st1|st49> --table-path <delta_table_path> [--csv-path <single_csv_file> | --csv-folder <folder_with_csvs>] [--log-path <log_file_path>] [--recreate-table]`

  - `--report-type`: Specify `st1` or `st49`.
  - `--table-path`: The path where your Delta table will be created or exists.
  - `--csv-path`: (Optional) Path to a single CSV file to load.
  - `--csv-folder`: (Optional) Path to a folder containing CSV files to load. Files are filtered by report type prefix (WELLS for ST1, SPUD for ST49).
  - `--log-path`: (Optional) Path to a log file to track processed CSVs (defaults to `delta_load_log.json`).
  - `--recreate-table`: (Optional) If present, the Delta table and log file will be deleted and recreated before loading.

  Example (loading a single CSV): `cargo run load-delta --report-type st1 --csv-path ./CSV/WELLS20230101.csv --table-path ./delta_tables/st1_data`
  Example (loading from a folder): `cargo run load-delta --report-type st49 --csv-folder ./CSV --table-path ./delta_tables/st49_data --recreate-table`

## License

This project is licensed under the MIT License