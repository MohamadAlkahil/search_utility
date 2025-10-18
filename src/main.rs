use colored::Colorize;
use regex::{Captures, Regex, RegexBuilder};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use walkdir::WalkDir;

//The Config Struct holds the data assocaited with the Command Line Argument
struct Config {
    pattern: String,
    file_paths: Vec<String>,
    case_insensitive: bool,
    print_line_numbers: bool,
    invert_match: bool,
    recursive_search: bool,
    print_filenames: bool,
    colored_output: bool,
    help: bool,
}


impl Config {
    // essentially the constructor for the Config struct
    fn new(args: &Vec<String>) -> Result<Self, String> {
        // default values set for config
        let mut config = Config {
            pattern: String::new(),
            file_paths: Vec::new(),
            case_insensitive: false,
            print_line_numbers: false,
            invert_match: false,
            recursive_search: false,
            print_filenames: false,
            colored_output: false,
            help: false,
        };
        //loop through args to try and update default values for option flags and non option flags get added to vector
        let mut non_options = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            // the first arg only stores program name so skip it
            if i == 0 {
                continue;
            } else {
                match arg.as_str() {
                    "-i" => config.case_insensitive = true,
                    "-n" => config.print_line_numbers = true,
                    "-v" => config.invert_match = true,
                    "-r" => config.recursive_search = true,
                    "-f" => config.print_filenames = true,
                    "-c" => config.colored_output = true,
                    "-h" | "--help" => config.help = true,
                    // anything that is not a flag must be related to file path or pattern so push to non_options vector to be dealt with later.
                    _ => non_options.push(arg.clone()),
                }
            }
        }
        if config.help {
            return Ok(config);
        }

        if non_options.is_empty() {
            return Err(String::from("Error: No pattern provided"));
        }
        if non_options.len() < 2 {
            return Err(String::from("Error: No file paths provided"));
        }
        // ASSUMPTION: pattern will not be empty and will be correctly be input ahead of file paths
        config.pattern = non_options[0].clone();
        config.file_paths.extend_from_slice(&non_options[1..]);
        // go through and find all file pths if recursive_search is set
        if config.recursive_search {
            config.file_paths = match recursively_find_all_files(&config.file_paths) {
                Ok(found_file_paths) => found_file_paths,
                Err(e) => return Err(e),
            }
        }
        Ok(config)
    }
}

/*
Breif Explanation: Finds all files in given directory.

Parameters: 
    directories: &Vec<String> - directory of all file paths to search.

Returns: 
    Ok(Vec<String>) - all files to be searched for pattern.
    Err(String) - Error mesage if recursion fails.
*/
fn recursively_find_all_files(directories: &Vec<String>) -> Result<Vec<String>, String> {
    let mut file_paths = Vec::new();
    for directory in directories {
        // get the metadata to see if file is actually a folder or not
        let metadata = match fs::metadata(directory) {
            Ok(metadata) => metadata,
            Err(_) => return Err(format!("Error: could not get metadata for: {}", directory)),
        };
        // if it is a file then push to the entire filepath to vector
        if metadata.is_file() {
            file_paths.push(directory.to_string());
        // if it is actually a directory walk through directory and push all files that are not hidden to the vector
        } else if metadata.is_dir() {
            for entry in WalkDir::new(directory) {
                match entry {
                    Ok(entry) => {
                        if entry.file_type().is_file() {
                            let file_name = entry.file_name().to_str().unwrap_or("");
                            if !file_name.starts_with(".") {
                                file_paths.push(entry.path().display().to_string());
                            }
                        }
                    }
                    Err(_) => return Err(format!("Error: could not read directory {}", directory)),
                }
            }
        }
    }
    Ok(file_paths)
}

fn main() {
    // get the the command line arguments and use them to intialize an instance of Config struct
    let passed_args: Vec<String> = env::args().collect();
    let config_set = match Config::new(&passed_args) {
        Ok(config) => config,
        Err(e) => {
            println!("{e}");
            display_help();
            return;
        }
    };
    // if the user entered a help option flag print the help message and exit
    if config_set.help {
        display_help();
        return;
    }
    // go through all file paths and search through the file to find matches
    for file_path in &config_set.file_paths {
        match search_file(&file_path, &config_set) {
            Ok(_) => (),
            Err(e) => {
                println!("{e}");
                return;
            }
        }
    }
}

fn display_help() {
    println!(
        "Usage: grep [OPTIONS] <pattern> <files...>

Options:
-i                Case-insensitive search
-n                Print line numbers
-v                Invert match (exclude lines that match the pattern)
-r                Recursive directory search
-f                Print filenames
-c                Enable colored output
-h, --help        Show help information"
    );
}

/*
Breif Explanation: Searches for pattern in a given file.

Parameters: 
    file_path: &String - the file path for a given file.
    config: &Config - instance of a config struct that holds search options.    

Returns: 
    Ok(()) - search done.
    Err(String) - Error mesage if searching a file fails.
*/
fn search_file(file_path: &String, config: &Config) -> Result<(), String> {
    let f = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(format!("Could not open file: {}", file_path)),
    };
    //buffer used to read a single line from the file at a time
    let buf_reader = BufReader::new(f);
    //regex created to try to find matches within a line
    //escapes all regular expression meta characters in pattern
    //case insensitity passed from config struct
    //unicode enabled to esnure valid UTF-8 matches
    let re = match RegexBuilder::new(&regex::escape(&config.pattern))
        .case_insensitive(config.case_insensitive)
        .unicode(true)
        .build()
    {
        Ok(re) => re,
        Err(_) => return Err(format!("Could not create regex builder for pattern")),
    };
    //go line by line and if no error is hit then prin the line and the associated data based on the config instance
    for (i, line_result) in buf_reader.lines().enumerate() {
        match line_result {
            Ok(line) => {
                let (pattern_found, display_line) =
                    pattern_in_line(&re, config.colored_output, &line);
                if should_print(config.invert_match, pattern_found) {
                    print_match(&config, &file_path, i + 1, &display_line)
                }
            }
            Err(_) => return Err(format!("Could not read line {} from {}", i + 1, file_path)),
        }
    }
    Ok(())
}

/*
Breif Explanation: Searches for pattern in a given line.

Parameters: 
    re: &Regex - the regex pattern.
    colored_output: bool - the option set if colored output is selected in search configuration.
    line: &String - the line to be searched.    

Returns: 
   (pattern_found: bool, display_line: &String):
        pattern_found - holds if pattern was found
        display_line - holds the line that was searched and if colored_output was selected then the matched pattern is replaced for red version.
*/
fn pattern_in_line(re: &Regex, colored_output: bool, line: &String) -> (bool, String) {
    // no match found so return as is
    if !re.is_match(line) {
        return (false, line.to_string());
    }
    //match found but not trying to color so return as is
    if !colored_output {
        return (true, line.to_string());
    }
    //match found but color needed

    //caps[0] will hold exact matches from the line
    //we use colorize crate to update color to red
    let replacement = |caps: &Captures| caps[0].red().to_string();
    //replace all non-overlapping matches in the line with the replacement
    let colored_line = re.replace_all(line, &replacement);
    (true, colored_line.to_string())
}

/*
Breif Explanation: Determines if a print is required based on if there was a match and inverted option was selected.

Parameters: 
    invert_match: bool - the option set if invert_match is selected in search configuration.
    pattern_found: bool - was the pattern found in the given line.

Returns: 
   true if line should be printed and false otherwise.
*/
fn should_print(invert_match: bool, pattern_found: bool) -> bool {
    if invert_match {
        !pattern_found
    } else {
        pattern_found
    }
}

/*
Breif Explanation: prints matched line and associated data.

Parameters: 
    config: &Config - instance of a config struct that holds search options.    
    file_path: &String - the file path for the associated file.
    line_number: usize - the associated line number in the file for the line.
    line: &String - the line to be printed.

Returns: NA
*/
fn print_match(config: &Config, file_path: &String, line_number: usize, line: &String) {
    let mut output_list = Vec::new();
    if config.print_filenames {
        output_list.push(file_path.to_string());
    }
    if config.print_line_numbers {
        output_list.push(line_number.to_string());
    }
    output_list.push(line.to_string());
    println!("{}", output_list.join(": "));
}
