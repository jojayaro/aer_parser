use aer_st1::st1;
use aer_st1::st49;
use once_cell::sync::Lazy;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

static INIT: Lazy<()> = Lazy::new(|| {
    env_logger::init();
});

fn setup() {
    Lazy::force(&INIT);
}

#[test]
fn it_works() {
    setup();
    assert_eq!(2 + 2, 4);
}

#[tokio::test]
async fn test_st1_processing_and_csv_output() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    // Define paths
    let _input_file_path = PathBuf::from("TXT/WELLS0102.TXT");
    let expected_output_file_path = PathBuf::from("tests/test_data/20140102_WELLS_expected.csv");
    let actual_output_file_path = PathBuf::from("CSV/20140102_WELLS.csv");

    // Ensure the CSV directory exists
    fs::create_dir_all(actual_output_file_path.parent().unwrap())?;

    // Process the ST1 file
    st1::process_file("WELLS0102", "TXT", "CSV").await?;

    // Read the content of the generated CSV and the expected CSV
    let mut actual_csv_content = String::new();
    File::open(&actual_output_file_path)?.read_to_string(&mut actual_csv_content)?;

    let mut expected_csv_content = String::new();
    File::open(&expected_output_file_path)?.read_to_string(&mut expected_csv_content)?;

    // Compare the contents
    assert_eq!(actual_csv_content.trim(), expected_csv_content.trim());

    // Clean up the generated CSV file
    fs::remove_file(&actual_output_file_path)?;

    Ok(())
}

#[tokio::test]
async fn test_st49_processing_and_csv_output() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    // Define paths
    let _input_file_path = PathBuf::from("TXT/SPUD0101.TXT");
    let expected_output_file_path = PathBuf::from("tests/test_data/20140101_SPUD_expected.csv");
    let actual_output_file_path = PathBuf::from("CSV/20140101_SPUD.csv");

    // Ensure the CSV directory exists
    fs::create_dir_all(actual_output_file_path.parent().unwrap())?;

    // Process the ST49 file
    st49::process_file("SPUD0101", "TXT", "CSV").await?;

    // Read the content of the generated CSV and the expected CSV
    let mut actual_csv_content = String::new();
    File::open(&actual_output_file_path)?.read_to_string(&mut actual_csv_content)?;

    let mut expected_csv_content = String::new();
    File::open(&expected_output_file_path)?.read_to_string(&mut expected_csv_content)?;

    // Compare the contents
    assert_eq!(actual_csv_content.trim(), expected_csv_content.trim());

    // Clean up the generated CSV file
    fs::remove_file(&actual_output_file_path)?;

    Ok(())
}
