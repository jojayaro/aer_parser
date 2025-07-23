#!/bin/bash
set -e
cd /home/sidefxs/repos/aer_parser

# Get yesterday's date in YYYY-MM-DD format
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)

# Download and process st1 files
aer_parser date-range --report-type st1 --start-date "$YESTERDAY" --end-date "$YESTERDAY" --txt-output-dir data/txt --csv-output-dir data/csv

# Load st1 CSVs into Delta table
aer_parser load-delta --report-type st1 --csv-folder data/csv --table-path data/deltalake/st1

# Download and process st49 files
aer_parser date-range --report-type st49 --start-date "$YESTERDAY" --end-date "$YESTERDAY" --txt-output-dir data/txt --csv-output-dir data/csv

# Load st49 CSVs into Delta table
aer_parser load-delta --report-type st49 --csv-folder data/csv --table-path data/deltalake/st49
