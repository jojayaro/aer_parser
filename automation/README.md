# aer_parser Automation Setup

This folder contains files to automate daily data download and Delta table loading for the aer_parser project using systemd on Rocky Linux.

## Files
- `run_daily_aer_parser.sh`: Shell script that runs the daily workflow. It calculates yesterday's date, downloads/processes st1 and st49 files, and loads them into Delta tables.
- `aer_parser_daily.service`: Systemd service file to run the shell script as the `sidefxs` user in the project directory.
- `aer_parser_daily.timer`: Systemd timer file to trigger the service daily at 2:00 AM.

## Setup Instructions
1. **Build the release binary:**
   ```bash
   cargo build --release
   ```
   The binary will be at `target/release/aer_parser`.

2. **Make the script executable:**
   ```bash
   chmod +x automation/run_daily_aer_parser.sh
   ```

3. **Copy service and timer files to systemd directory:**
   ```bash
   sudo cp automation/aer_parser_daily.service automation/aer_parser_daily.timer /etc/systemd/system/
   ```

4. **Reload systemd and enable the timer:**
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable aer_parser_daily.timer
   sudo systemctl start aer_parser_daily.timer
   ```

5. **Check logs:**
   ```bash
   journalctl -u aer_parser_daily.service
   ```

## Notes
- The service runs as `sidefxs` and uses `/home/sidefxs/repos/aer_parser` as the working directory.
- The script uses yesterday's date for data download and loading.
- Adjust the timer (`OnCalendar`) in the timer file if you want to change the run time.
- For troubleshooting, check the logs and ensure the release binary is up to date.
