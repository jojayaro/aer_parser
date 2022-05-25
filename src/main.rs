use std::env;
//use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead};
//use csv::Writer;


fn main() {
    //Collect arguments when running the program
    let args: Vec<String> = env::args().collect();

    let query = &args[1];
    let filename = &args[2];

    println!("Searching for {}", query);
    println!("In file {}", filename);

//    //Open File
//    let contents = fs::read_to_string(filename)
//        .expect("Something went wrong reading the file");

    //println!("With text:\n{}", contents);

    //Search for indices where query is found
    //let v1: Vec<_> = contents.match_indices(query).collect();

    //println!("{:?}", v1);

    //Insert file lines contents into a vector
    let file = File::open(filename).expect("File not found");
    let content = BufReader::new(file);

    let lines: Vec<String> = content
        .lines()
        .map(|line| line.expect("Something went wrong"))
        .collect();

    //Iterate over lines to index the position where the query is found
    let lines_iter = lines.iter().enumerate();

    let mut lines_index_break: Vec<usize> = Vec::new();
    let mut lines_index_date: Vec<usize> = Vec::new();
    
    for (pos, e) in lines_iter {
        if e.contains("---") {
            lines_index_break.push(pos);
            //println!("Element at position {}: {:?}", pos, e);
        } else if e.contains("DATE") {
            lines_index_date.push(pos);
            //println!("Element at position {}: {:?}", pos, e);
        }

        };

        let date = &lines[lines_index_date[0]].trim()[6..];
        println!("{}", date);

    //Slice Lines vector to include only licences
    let mut licences_vec: Vec<&str> = Vec::new();
    
    for i in lines_index_break[1]+1..lines_index_break[2]-2 {
        if lines[i].trim().len() > 0 {
            licences_vec.push(&lines[i].trim());
        }
    };

    let mut licence_vec_clean_split: Vec<&str> = Vec::new();

    let mut i = 0;

    while i < licences_vec.len() {
        licence_vec_clean_split.push(&licences_vec[i].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i].trim()[37..47]);
        licence_vec_clean_split.push(&licences_vec[i].trim()[47..68]);
        licence_vec_clean_split.push(&licences_vec[i].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+1].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i+1].trim()[37..47]);
        licence_vec_clean_split.push(&licences_vec[i+1].trim()[47..68]);
        licence_vec_clean_split.push(&licences_vec[i+1].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+2].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i+2].trim()[37..68]);
        licence_vec_clean_split.push(&licences_vec[i+2].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+3].trim()[..37]);
        licence_vec_clean_split.push(&licences_vec[i+3].trim()[37..47]);
        licence_vec_clean_split.push(&licences_vec[i+3].trim()[47..68]);
        licence_vec_clean_split.push(&licences_vec[i+3].trim()[68..]);

        licence_vec_clean_split.push(&licences_vec[i+4].trim()[..68]);
        licence_vec_clean_split.push(&licences_vec[i+4].trim()[68..]);

        i += 5;
    };


    //println!("{:?}", licence_vec_clean_split);

    //Find empty lines in the licences vector
 //   let mut lines_index_empty: Vec<usize> = Vec::new();
 //   
 //   for (pos, e) in licences_vec.iter().enumerate() {
 //       if e.len() == 0 {
 //           lines_index_empty.push(pos);
 //           //println!("Element at position {}: {:?}", pos, e);
//
 //       }
 //   };
 //   
 //   //Remove empty lines from the licences vector
 //   let licences_vec_clean = licences_vec.into_iter().filter(|&i| i.len() != 0).collect::<Vec<_>>();
//
 //   //Iterate over licences_vec_clean vector and split every element by whitespace and push results into a new vector
//    let mut licence_vec_clean_split: Vec<&str> = Vec::new();
//
//    for i in licences_vec {
//        let mut split_vec: Vec<&str> = i.split("  ").collect();
//        licence_vec_clean_split.append(&mut split_vec);
//    };
//    
//    let licence_vec_clean_split = licence_vec_clean_split.into_iter().filter(|&i| i.len() != 0).collect::<Vec<_>>();

    let number_licences = licence_vec_clean_split.len()/17;

    let path = format!("{}.csv", date);

    //create a csv with the results from licence_vec_clean_split vector divided by 18 columns and write a file
    let csv_file = File::create(path).expect("Unable to create file");
    let mut csv_writer = csv::Writer::from_writer(csv_file);

    for i in 0..number_licences {
        let mut row = Vec::new();
        for j in 0..17 {
            row.push(licence_vec_clean_split[i*17+j].trim());
        }
        row.push(date);

        //println!("{:?}", row);

        csv_writer.write_record(&row).expect("Unable to write to file");
    }

    csv_writer.flush().expect("Unable to flush to file");




    //Print to check

    //println!("{:?}", lines_index_break);

    //println!("{:?}", lines_index_empty);
   
    //println!("{:?}", lines);
}
