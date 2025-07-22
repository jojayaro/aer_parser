# CLI Rules & Development Guidelines

This document outlines the development practices and usage guidelines for the `aer_parser` command-line interface.

## Core Development Principles

### Code Quality & Maintenance
*   **Small, Incremental Changes:** Make focused, atomic changes that are easy to review and rollback
*   **Code Deduplication:** Identify and eliminate duplicate code through shared utilities and modules
*   **Memory Efficiency:** Use streaming/buffered operations for large files to minimize memory usage
*   **Performance First:** Profile before optimizing; maintain benchmarks for critical paths
*   **Backward Compatibility:** Preserve existing CLI interface and file format support

### Testing Strategy
*   **Test-Driven Development (TDD):** Write tests before implementing new features or fixing bugs
*   **Comprehensive Testing:** Maintain 90%+ test coverage with unit, integration, and property tests
*   **Test Data Management:** Use fixtures and mock data instead of hardcoded paths
*   **Performance Testing:** Include benchmarks for parsing operations and memory usage
*   **Regression Testing:** Run full test suite before each commit

### Code Organization
*   **Modular Design:** Separate concerns into distinct modules (parsing, I/O, error handling)
*   **Shared Utilities:** Extract common functionality into reusable components
*   **Documentation Standards:** Document all public APIs with examples and usage patterns
*   **Error Context:** Provide detailed error messages with file context and recovery suggestions

## Development Workflow

### Pre-commit Checklist
```bash
# Run before each commit
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo test --test integration_tests
```

### Performance Validation
```bash
# Run performance benchmarks
cargo bench
# Check memory usage
cargo test --release -- --nocapture | grep "Memory usage"
```

## Error Handling & Logging Standards

### Error Types
*   **Use `thiserror` for custom error types** with rich context
*   **Provide actionable error messages** with file positions and expected formats
*   **Implement error recovery** where possible (skip bad records, continue processing)

### Logging Guidelines
*   **INFO**: High-level progress (file processing start/end, record counts)
*   **DEBUG**: Detailed parsing information (field extraction, format detection)
*   **WARN**: Recoverable issues (skipped records, format variations)
*   **ERROR**: Critical failures with context

## Testing Standards

### Test Structure
```
tests/
├── unit/           # Individual function tests
├── integration/    # End-to-end workflows
├── fixtures/       # Test data files
└── benchmarks/     # Performance tests
```

### Test Data Guidelines
*   **Use relative paths** from project root
*   **Create temporary files** for output testing
*   **Include edge cases**: empty files, malformed data, large files
*   **Property-based testing** for format validation

## Performance Optimization

### Memory Management
*   **Buffered reading** for large files (>10MB)
*   **Streaming CSV writing** to reduce memory peaks
*   **String interning** for repeated values (licenses, company names)
*   **Progress reporting** for long-running operations

### Benchmarking
```bash
# Run parsing benchmarks
cargo bench --bench parsing_benchmarks
# Memory profiling
cargo test --release -- --nocapture
```

## Documentation Standards

### Code Documentation
*   **Module-level docs** with usage examples
*   **Function-level docs** with parameter descriptions
*   **Inline comments** for complex parsing logic
*   **Error documentation** with recovery strategies

### README Maintenance
*   **Keep examples current** with latest CLI syntax
*   **Add troubleshooting section** for common issues
*   **Include performance tips** for large datasets
*   **Document breaking changes** in CHANGELOG.md

## Refactoring Guidelines

### When to Refactor
*   **Code duplication** > 20 lines
*   **Function complexity** > 50 lines
*   **Cyclomatic complexity** > 10
*   **Repeated error patterns**

### Refactoring Process
1.

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
