use colored::Colorize;
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

        if non_options.is_empty() {
            return Err(String::from("Error: No pattern provided"));
        }
        if non_options.len() < 2 {
            return Err(String::from("Error: No file paths provided"));
        }
        // ASSUMPTION: pattern will correctly be input ahead of file paths
        config.pattern = non_options[0].clone();
        if config.case_insensitive {
            config.pattern = config.pattern.to_lowercase();
        }
        config.file_paths.extend_from_slice(&non_options[1..]);
        if config.recursive_search {
            config.file_paths = match recursively_find_all_files(&config.file_paths) {
                Ok(found_file_paths) => found_file_paths,
                Err(e) => return Err(e),
            }
        }
        Ok(config)
    }
}

fn recursively_find_all_files(directories: &Vec<String>) -> Result<Vec<String>, String> {
    let mut file_paths = std::collections::HashSet::new();
    for directory in directories {
        let metadata = match fs::metadata(directory) {
            Ok(metadata) => metadata,
            Err(_) => return Err(format!("Error: could not get metadata for: {}", directory)),
        };
        if metadata.is_file() {
            file_paths.insert(directory.to_string());
        } else if metadata.is_dir() {
            for entry in WalkDir::new(directory) {
                match entry {
                    Ok(entry) => {
                        if entry.file_type().is_file() {
                            file_paths.insert(entry.path().display().to_string());
                        }
                    }
                    Err(_) => return Err(format!("Error: could not read directory {}", directory)),
                }
            }
        }
    }
    Ok(file_paths.into_iter().collect())
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

fn search_file(file_path: &String, config: &Config) -> Result<(), String> {
    let f = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(format!("Could not open file: {}", file_path)),
    };
    //buffer used to read a single line from the file at a time
    let buf_reader = BufReader::new(f);
    //go line by line and if no error is hit then prin the line and the associated data based on the config instance
    for (i, line_result) in buf_reader.lines().enumerate() {
        match line_result {
            Ok(line) => {
                let (pattern_found, display_line) = pattern_in_line(
                    config.case_insensitive,
                    config.colored_output,
                    &line,
                    &config.pattern,
                );
                if should_print(config.invert_match, pattern_found) {
                    print_match(&config, &file_path, i + 1, &display_line)
                }
            }
            Err(_) => return Err(format!("Could not read line {} from {}", i + 1, file_path)),
        }
    }
    Ok(())
}

fn pattern_in_line(
    case_insensitive: bool,
    colored_output: bool,
    line: &String,
    pattern: &String,
) -> (bool, String) {
    if case_insensitive {
        contains(&line.to_lowercase(), &line, pattern, colored_output)
    } else {
        contains(&line, &line, pattern, colored_output)
    }
}

fn contains(
    search_line: &String,
    original_line: &String,
    pattern: &String,
    colored_output: bool,
) -> (bool, String) {
    let pattern_index_found = search_line.find(pattern);
    if pattern_index_found.is_some() {
        if !colored_output {
            return (true, original_line.to_string());
        } else {
            let mut updated_line = original_line.to_string();
            let i = pattern_index_found.unwrap();
            let pattern_colorized = original_line[i..(i + pattern.len())].red().to_string();
            updated_line.replace_range(i..(i + pattern.len()), &pattern_colorized[..]);
            return (true, updated_line);
        }
    }
    (false, original_line.to_string())
}

fn should_print(invert_match: bool, pattern_found: bool) -> bool {
    if invert_match {
        !pattern_found
    } else {
        pattern_found
    }
}

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
