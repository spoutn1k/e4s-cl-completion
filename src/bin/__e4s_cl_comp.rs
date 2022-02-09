use dirs::home_dir;
use e4s_cl_completion::ex::SAMPLE_JSON;
use e4s_cl_completion::structures::{ArgumentCount, Command, Completable, Option_, Profile};
use std::convert::TryFrom;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

static ENV_LINE_VAR: &str = "COMP_LINE";

static DATABASE: &'static str = ".local/e4s_cl/user.json";

fn get_subcommand<'a>(desc: &'a Command, name: &str) -> Option<&'a Command> {
    desc.subcommands.iter().find(|c| c.name.as_str() == name)
}

fn get_option<'a>(desc: &'a Command, name: &str) -> Option<&'a Option_> {
    for option in desc.options.iter() {
        if option.names.iter().find(|x| x.as_str() == name).is_some() {
            return Some(option);
        }
    }

    None
}

fn read_profiles<P: AsRef<Path>>(path: P) -> Result<Vec<Profile>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file.
    let data: serde_json::Value = serde_json::from_reader(reader)?;

    match data["Profile"].as_object() {
        Some(map) => Ok(map
            .iter()
            .map(|(_i, data)| serde_json::from_value::<Profile>(data.to_owned()).unwrap())
            .collect()),
        None => Ok(vec![]),
    }
}

fn load_example() -> Result<Command, Box<dyn Error>> {
    Ok(serde_json::from_str(SAMPLE_JSON)?)
}

fn routine(arguments: &Vec<String>) {
    let root_command: Command;

    match load_example() {
        Ok(object) => root_command = object,
        Err(error) => {
            eprintln!("Error loading JSON: {}", error);
            return;
        }
    }

    let candidates: Vec<&str>;

    let db_file = home_dir().unwrap().join(DATABASE);
    let profiles: Vec<Profile> = read_profiles(db_file).unwrap_or(vec![]);

    let empty_option = Option_ {
        names: vec![],
        values: vec![],
        arguments: ArgumentCount::Fixed(0),
        expected_type: String::from(""),
    };

    let mut pos = 1;
    let mut current_command: &Command = &root_command;
    let mut current_option = &empty_option;
    while pos < arguments.len() {
        let token = &arguments[pos];

        if token.len() == 0 {
            pos += 1;
            continue;
        }

        // Check if the token introduces a new subcommand
        match get_subcommand(&current_command, &token) {
            Some(command) => {
                // Switch context over to the new command, and start again
                current_command = command;
                current_option = &empty_option;
                pos += 1;
                continue;
            }
            None => (),
        }

        match get_option(&current_command, &token) {
            // Get the option if it exists
            Some(option) => {
                if let ArgumentCount::Fixed(count) = option.arguments {
                    // n_args > 0
                    let n_args = usize::try_from(count).unwrap();
                    // If the expected arguments are on the CLI, skip them
                    if pos + n_args < arguments.len() - 1 {
                        pos += n_args;
                        continue;
                    } else {
                        current_option = option;
                    }
                }
            }
            None => (),
        }

        pos += 1;
    }

    if current_option.names.is_empty() {
        candidates = current_command.candidates(&profiles);
    } else {
        candidates = current_option.candidates(&profiles);
    }

    for completion in candidates.iter() {
        if completion.starts_with(arguments.last().unwrap()) {
            println!("{}", completion);
        }
    }
}

fn main() {
    let mut args = env::args();
    let command_line: Vec<String>;

    match std::env::var(&ENV_LINE_VAR) {
        Ok(string) => command_line = string.split(" ").map(|s| s.to_string()).collect(),
        Err(_) => {
            println!(include_str!("complete.fmt"), args.next().unwrap());
            exit(0);
        }
    }

    routine(&command_line)
}
