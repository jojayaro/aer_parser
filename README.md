# ST1 Alberta Energy Regulator Parser

## Overview

This high-performance Rust-based parser efficiently processes the Alberta Energy Regulator's ST1 data, which contains critical information on oil and gas exploration licenses. The ST1 report, available from the AER's [official page](https://www.aer.ca/providing-information/data-and-reports/statistical-reports/st1), provides valuable insights into the regulatory landscape of Alberta's energy sector.

## Key Features

- **Asynchronous File Downloading**: Utilizes `tokio` and `reqwest` for efficient, non-blocking file retrieval.
- **Robust Parsing Algorithm**: Implements a sophisticated parsing mechanism to extract structured data from raw text files.
- **CSV Output**: Generates clean, analysis-ready CSV files for seamless integration with data processing pipelines.
- **Error Handling**: Comprehensive error management for resilient operation.
- **Memory Efficient**: Employs Rust's ownership model for optimal memory usage when processing large datasets.

## Usage

### File Retrieval

The parser includes a commented function for downloading individual files using the MMDD format. To activate this feature, uncomment the relevant code in `src/main.rs`.

### Local Parsing

To parse a locally stored file:

- Process a single file: `cargo run file WELLS20230101.TXT`
- Process all files in a folder: `/path/to/your/folder`
- Download and process files for a date range: `cargo run date_range 2023-01-01 2023-01-31`

## License

This project is licensed under the MIT License



