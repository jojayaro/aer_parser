//! Test fixtures and utilities for AER parser testing
//!
//! This module provides test data, fixtures, and utilities for comprehensive
//! testing of the AER parser without hardcoded paths.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

/// Test data directory structure
pub struct TestData {
    pub temp_dir: TempDir,
    pub st1_valid: PathBuf,
    pub st1_invalid: PathBuf,
    pub st49_valid: PathBuf,
    pub st49_invalid: PathBuf,
    pub st1_edge_cases: PathBuf,
    pub st49_edge_cases: PathBuf,
}

impl TestData {
    /// Create test data directory structure
    pub fn new() -> Result<Self, std::io::Error> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        let st1_valid = root.join("st1/valid");
        let st1_invalid = root.join("st1/invalid");
        let st49_valid = root.join("st49/valid");
        let st49_invalid = root.join("st49/invalid");
        let st1_edge_cases = root.join("st1/edge_cases");
        let st49_edge_cases = root.join("st49/edge_cases");

        // Create directories
        for dir in &[&st1_valid, &st1_invalid, &st49_valid, &st49_invalid, &st1_edge_cases, &st49_edge_cases] {
            fs::create_dir_all(dir)?;
        }

        Ok(TestData {
            temp_dir,
            st1_valid,
            st1_invalid,
            st49_valid,
            st49_invalid,
            st1_edge_cases,
            st49_edge_cases,
        })
    }

    /// Create a sample ST1 file
    pub fn create_st1_sample(&self, filename: &str, date: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.st1_valid.join(filename);
        let content = format!(
            r#"ALBERTA ENERGY REGULATOR
DAILY WELL LICENCES LIST
DATE: {date}

WELL LICENCES ISSUED
--------------------------------------------------------------------------------------------
WELL NAME                           LICENCE NUMBER  MINERAL RIGHTS              GROUND ELEV
UNIQUE IDENTIFIER                   SURFACE COORD   AER FIELD CENTRE            PROJECTED DEPTH
AER CLASSIFICATION                  FIELD           TERMINATING ZONE            
DRILLING OPERATION                  WELL PURPOSE    WELL TYPE       SUBSTANCE   
LICENSEE                            SURFACE LOCATION
--------------------------------------------------------------------------------------------
WELL 1                              123456          FREEHOLD                    1000
UID-001                             12-34-56-01W4   CALGARY                     3000
NEW                                 FIELD-A         BANFF                       
HORIZONTAL                          PRODUCTION      GAS WELL        GAS         
COMPANY A                           12-34-56-01W4
WELL 2                              123457          CROWN                       950
UID-002                             12-34-56-02W4   CALGARY                     2800
NEW                                 FIELD-B         LEDUC                       
VERTICAL                            PRODUCTION      OIL WELL        OIL         
COMPANY B                           12-34-56-02W4
--------------------------------------------------------------------------------------------
END OF WELL LICENCES DAILY LIST
"#
        );
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Create a sample ST49 file
    pub fn create_st49_sample(&self, filename: &str, date: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.st49_valid.join(filename);
        let content = format!(
            "AER DAILY SPUD REPORT {}\n\
            --------------------------------------------------------------------------------------------\n\
            WELL ID     WELL NAME           LICENCE     BA ID   NAME            NUMBER  ACTIVITY DATE\n\
            FIELD CENTRE BA ID   LICENSEE                NEW PROJECTED TOTAL DEPTH ACTIVITY TYPE\n\
            --------------------------------------------------------------------------------------------\n\
            W001        WELL-A              123456      1001    CONTRACTOR-A    RIG-001 2024-01-01\n\
            CALGARY     2001    COMPANY-A               3000                    SPUD\n\
            W002        WELL-B              123457      1002    CONTRACTOR-B    RIG-002 2024-01-01\n\
            CALGARY     2002    COMPANY-B               2800                    SPUD\n\
            --------------------------------------------------------------------------------------------\n\
            Report Number: ST-49\n\
            Run Date: 2024-01-01\n\
            For the Notification Period: 2024-01-01\n\
            TOTAL  - 2\n\
            END OF REPORT\n",
            date
        );
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Create an empty ST1 file
    pub fn create_empty_st1(&self, filename: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.st1_edge_cases.join(filename);
        fs::write(&path, "")?;
        Ok(path)
    }

    /// Create a malformed ST1 file
    pub fn create_malformed_st1(&self, filename: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.st1_invalid.join(filename);
        let content = "INVALID FILE FORMAT\nNO DATE LINE\nNO DATA SECTION";
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Create an ST1 file with missing sections
    pub fn create_missing_sections_st1(&self, filename: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.st1_edge_cases.join(filename);
        let content = "ALBERTA ENERGY REGULATOR\nDAILY WELL LICENCES LIST\nDATE: 02 January 2024\n\nNO DATA SECTION HERE";
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Create an ST49 file with invalid date format
    pub fn create_invalid_date_st49(&self, filename: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.st49_invalid.join(filename);
        let content = "AER DAILY SPUD REPORT INVALID DATE\nINVALID FORMAT";
        fs::write(&path, content)?;
        Ok(path)
    }
}
