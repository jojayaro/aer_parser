# ST1 Alberta Energy Regulator Parser in Rust

This is a parser for the Alberta Energy Regulator's ST1 data written in Rust. The ST1 report provides information on licenses awarded by the AER to companies for oil and gas exploration. The data is available in text files from the AER's [ST1 page](https://www.aer.ca/providing-information/data-and-reports/statistical-reports/st1).

Parser has a commented function to download an individual file using the MMDD format, or you can download the file and run the parser on it locally using the following command, where MMDD is month and day of the report (NOTE this is only available for the current year):

    cargo run --release -- MMDD

