# Copilot Instructions

This document provides guidance for AI agents working on the `aer_st1` codebase.

## Project Overview

This is a Rust command-line application designed to parse oil and gas exploration license data from the Alberta Energy Regulator's (AER) ST1 statistical reports. It downloads fixed-width text files (`.TXT`), parses them, and outputs the structured data into CSV files.

The core logic resides entirely in `src/main.rs`.

## Architecture and Data Flow

1.  **Input**: The application can be initiated via three command-line arguments:
    *   `file`: Processes a single local `.TXT` file.
    *   `folder`: Processes all `.TXT` files in a specified directory.
    *   `date_range`: Downloads files for a given date range and then processes them.

2.  **Downloading**: The `download_file` and `download_files_by_date_range` async functions use `reqwest` and `tokio` to fetch data from the AER website (`https://static.aer.ca/prd/data/well-lic/WELLS{MMDD}.TXT`). Downloaded files are saved in the `TXT/` directory.

3.  **Parsing**: The `process_file` function orchestrates the parsing.
    *   It reads the `.TXT` file line by line.
    *   `Indices::search` finds the key structural markers in the file: the date line and the data section separators (`---`).
    *   `extract_licences_lines` isolates the lines containing the actual license data.
    *   `extract_license` is the core parsing function. It processes 5 lines at a time, extracting data from fixed-width columns using string slicing (`get(start..end)`). The layout is rigid and specific to the AER ST1 format.

4.  **Output**: The parsed data, held in a `Vec<License>`, is serialized into a CSV file in the `CSV/` directory using the `csv` crate. The output filename mirrors the input filename (e.g., `WELLS0101.TXT` becomes `CSV/WELLS0101.csv`).

## Key Files and Structs

-   `src/main.rs`: Contains the entire application logic.
-   `License`: The `struct` that defines the schema for the output data. Any changes to the data being extracted must be reflected here.
-   `Indices`: A helper `struct` for locating important lines within the source `.TXT` file before parsing.

## Developer Workflows

### Building the Project

Use the standard Rust command:
```sh
cargo build
```

### Running the Parser

The application is run via `cargo run` with specific subcommands and arguments.

-   **Process a single file from the `TXT` directory:**
    ```sh
    cargo run file WELLS0101.TXT
    ```
    *(Note: The code prepends `TXT/` if a simple filename is provided)*

-   **Process all files in a local folder (e.g., `TXT/`):**
    ```sh
    cargo run folder TXT
    ```

-   **Download and process files for a date range within the current year:**
    ```sh
    cargo run date_range 2025-01-01 2025-01-31
    ```

## Project-Specific Conventions

-   **Parsing Logic is Fragile**: The parsing in `extract_license` is tightly coupled to the fixed-width format of the source `.TXT` files. Any change in the AER's report format will break the parser. When modifying this, be mindful of the 5-line record structure and the exact character indices for each field.
-   **Error Handling**: The application uses `Result<T, Box<dyn std::error::Error>>` for error handling. Errors are generally propagated up to `main`.
-   **Dependencies**: Key external dependencies are `tokio` for async operations, `reqwest` for HTTP requests, `csv` for serialization, and `chrono` for date handling. These are defined in `Cargo.toml`.
