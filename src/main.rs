use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
        config.file_paths.extend_from_slice(&non_options[1..]);
        Ok(config)
    }
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
                if line.contains(&config.pattern) {
                    if config.print_filenames {
                        print!("{}: ", &file_path);
                    }
                    if config.print_line_numbers {
                        print!("{}: ", i + 1);
                    }
                    println!("{}", line);
                }
            }
            Err(_) => return Err(format!("Could not read line {} from {}", i + 1, file_path)),
        }
    }
    Ok(())
}
