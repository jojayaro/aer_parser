# Copilot Instructions

This document provides guidance for AI agents working on the `aer_parser` codebase.

## Project Overview

This is a Rust command-line application designed to parse oil and gas exploration license data from the Alberta Energy Regulator's (AER) statistical reports. It supports both ST1 (Well Licence) and ST49 (Spud) reports. It downloads fixed-width text files (`.TXT`), parses them, and outputs the structured data into CSV files.

The application is structured as a library crate with a binary executable, promoting modularity and separation of concerns.

## Architecture and Data Flow

1.  **Input**: The application is controlled via command-line arguments, specifying the report type (`st1` or `st49`) and the desired action (`file`, `folder`, or `date_range`).

2.  **Downloading (`src/downloader.rs`)**:
    *   The `download_files_by_date_range` function fetches files concurrently for a given date range.
    *   It uses `reqwest` for HTTP requests and `futures::future::join_all` for parallel execution, significantly speeding up the download process.
    *   It dynamically constructs the URL based on the report type:
        *   **ST1**: `https://static.aer.ca/prd/data/well-lic/WELLS{MMDD}.TXT` (uppercase extension)
        *   **ST49**: `https://static.aer.ca/prd/data/wells/SPUD{MMDD}.txt` (lowercase extension)
    *   Downloaded files are saved in the `TXT/` directory.

3.  **Parsing (`src/st1.rs`, `src/st49.rs`)**:
    *   Each report type has its own dedicated parsing module.
    *   **`st1.rs`**: Parses the 5-line records of the ST1 well licence reports.
    *   **`st49.rs`**: Parses the single-line records of the ST49 spud reports.
    *   Both parsers extract data from fixed-width columns using string slicing.

4.  **Output**: The parsed data, held in a `Vec<License>` (for ST1) or `Vec<SpudData>` (for ST49), is serialized into a CSV file in the `CSV/` directory using the `csv` crate.

## Key Files and Structs

-   `src/main.rs`: The binary entry point. Parses command-line arguments and calls the appropriate functions from the library.
-   `src/lib.rs`: The library root. Defines the `ReportType` enum and orchestrates the calls to the downloader and parser modules.
-   `src/error.rs`: Defines the custom `AppError` enum for centralized error handling using `thiserror`.
-   `src/downloader.rs`: Contains the parallel download logic.
-   `src/st1.rs`: Contains the `License` struct and all parsing logic for ST1 reports.
-   `src/st49.rs`: Contains the `SpudData` struct and all parsing logic for ST49 reports.

## Developer Workflows

### Building the Project

Use the standard Rust command:
```sh
cargo build
```

### Running the Parser

The application is run via `cargo run` with the report type, a command, and its arguments.

-   **Process a single ST49 file:**
    ```sh
    cargo run st49 file TXT/SPUD0101.TXT
    ```

-   **Process all ST1 files in a folder:**
    ```sh
    cargo run st1 folder TXT
    ```

-   **Download and process ST49 files for a date range:**
    ```sh
    cargo run st49 date_range 2025-01-01 2025-01-31
    ```

## Project-Specific Conventions

-   **Parsing Logic is Fragile**: The parsing logic is tightly coupled to the fixed-width format of the source files. Any change in the AER's report formats will likely break the parser.
-   **Error Handling**: The application uses a custom `AppError` enum and the `thiserror` crate for robust and descriptive error handling.
-   **Dependencies**: Key external dependencies are `tokio`, `reqwest`, `futures`, `csv`, `chrono`, and `thiserror`. These are defined in `Cargo.toml`.
