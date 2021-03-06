extern crate clap;
extern crate indicatif;
#[macro_use]
extern crate json;

use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use json::JsonValue;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process;

// Start of the command line tool
fn main() {
    // Build the args list
    let matches = App::new("csv-to-json")
        .version("0.1.0")
        .author("Justin Rhoades")
        .about("Convert csv to json")
        .arg(
            Arg::with_name("csv")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("path to csv file"),
        )
        .arg(
            Arg::with_name("output")
                .help("Sets path to output file")
                .short("o")
                .long("output")
                .value_name("FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("lower_case")
                .help("Sets the headers to lower case")
                .short("l")
                .long("lowercase")
                .multiple(false),
        )
        .arg(
            Arg::with_name("pretty")
                .help("Sets if the json made pretty or not")
                .short("p")
                .long("pretty")
                .multiple(false),
        )
        .get_matches();

    println!("Getting inputs");
    // Capturing the input arg
    let input: &str = match matches.value_of("csv") {
        Some(input) => input,
        None => {
            println!("Unable to read input");
            process::exit(1);
        }
    };

    // Capturing the output arg if given
    let output: String = match matches.value_of("output") {
        Some(output) => output.to_owned(),
        None => get_output(&input),
    };

    // See if we will be lower casing the header info
    let lower_case: bool = matches.is_present("lower_case");

    // See if we are building the json pretty or not
    let pretty: bool = matches.is_present("pretty");

    // Count
    let count: u64 = get_count(&input);

    println!("Reading csv file");
    // Open the input file
    let file = match File::open(input) {
        Ok(file) => file,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };
    let buf_reader = BufReader::new(file);

    // Create a json body
    let mut json_body: JsonValue;

    if count > 1 {
        json_body = array![];
    } else {
        json_body = object! {};
    }

    // Create reader and extract the header information
    let mut rdr = csv::Reader::from_reader(buf_reader);
    let headers = match rdr.headers() {
        Ok(headers) => headers.clone(),
        Err(err) => {
            println!("Error occured: {}", err);
            process::exit(1);
        }
    };

    // Set the progress bar
    let bar = ProgressBar::new(count);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
            )
            .progress_chars("#>-"),
    );

    println!("Processing records");
    // Loop over the records and create an object per record and add it to the array
    for result in rdr.records() {
        bar.inc(1);
        let record = result.unwrap();
        let mut element = object! {};
        for index in 0..headers.len() {
            if index >= record.len() {
                break;
            }

            let header: &str = &headers[index];
            let value: &str = &record[index];

            if value.is_empty() {
                if lower_case {
                    element[header.to_lowercase()] = json::Null;
                } else {
                    element[header] = json::Null;
                }
            } else {
                if lower_case {
                    element[header.to_lowercase()] = value.trim().into();
                } else {
                    element[header] = value.trim().into();
                }
            }
        }
        if count > 1 {
            match json_body.push(element.clone()) {
                Ok(_) => (),
                Err(err) => {
                    println!("Error: {}", err);
                    process::exit(1);
                }
            };
        } else {
            json_body = element;
            break;
        }
    }

    // Complete the progress bar
    bar.finish_with_message("Conversion completed");

    // Checking to see if path exists
    println!("Building path if needed");
    if !Path::new(&output.clone()).exists() {
        match fs::create_dir_all(Path::new(&output.clone()).parent().unwrap()) {
            Ok(_) => (),
            Err(err) => {
                println!("error while building directories: {}", err);
                process::exit(1)
            }
        };
    }

    println!("Writing JSON file now");

    // Output path
    let output_path = format!("{}", output);

    // Write the json data to the file
    if !pretty {
        match fs::write(output, json::stringify(json_body)) {
            Ok(_) => (),
            Err(err) => {
                println!("Error writing json: {}", err);
                process::exit(1);
            }
        }
    } else {
        match fs::write(output, json::stringify_pretty(json_body, 4)) {
            Ok(_) => (),
            Err(err) => {
                println!("Error writing json: {}", err);
                process::exit(1);
            }
        };
    }

    println!("JSON creationg at {}", output_path);
}

fn get_count(input: &str) -> u64 {
    // Go open the file again to generate another reader
    let file = match File::open(input) {
        Ok(file) => file,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };
    let buf_reader = BufReader::new(file);
    let mut rdr = csv::Reader::from_reader(buf_reader);

    // Create a count variable based on the record count
    let count: u64 = rdr.records().count() as u64;

    // Return count
    count
}

fn get_output(input: &str) -> String {
    let mut path_buff: std::path::PathBuf = PathBuf::from(input);
    path_buff.set_extension("json");
    let path_name: &str = path_buff.file_name().unwrap().to_str().unwrap();
    format!("{}", path_name)
}
