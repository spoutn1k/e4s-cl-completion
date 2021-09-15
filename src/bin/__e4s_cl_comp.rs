use dirs::home_dir;
use e4s_cl_completion::ex::SAMPLE_JSON;
use e4s_cl_completion::structures::{Command, Completable, Option_, Profile};
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

static UNSPECIFIED: i32 = -1;

static DATABASE: &'static str = ".local/e4s_cl/user.json";

fn get_subcommand<'a>(desc: &'a Command, name: &str) -> Option<&'a Command> {
    for command in desc.subcommands.iter() {
        if command.name.as_str() == name {
            return Some(command);
        }
    }

    None
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

fn routine() {
    let root_command = load_example().unwrap();

    let env_line_var: &str = "COMP_LINE";
    let args: Vec<String>;

    let mut candidates: Vec<String>;

    let db_file = home_dir().unwrap().join(DATABASE);
    let profiles: Vec<Profile> = read_profiles(db_file).unwrap_or(vec![]);

    match std::env::var(&env_line_var) {
        Ok(string) => args = string.split(" ").map(|s| s.to_string()).collect(),
        Err(string) => {
            eprintln!("Error accessing {}: {}", env_line_var, string);
            exit(1);
        }
    }

    let empty_option = Option_ {
        names: vec![],
        values: vec![],
        arguments: 0,
    };

    let mut pos = 1;
    let mut current_command: &Command = &root_command;
    let mut current_option = &empty_option;
    while pos != args.len() {
        let token = &args[pos];

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
                if option.arguments != UNSPECIFIED {
                    // n_args > 0
                    let n_args = usize::try_from(option.arguments).unwrap();
                    // If the expected arguments are on the CLI, skip them
                    if pos + n_args < args.len() - 1 {
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
        candidates.extend(
            current_command
                .subcommands
                .iter()
                .map(|x| x.name.to_owned())
                .collect::<Vec<String>>(),
        );
        candidates.extend(
            current_command
                .options
                .iter()
                .map(|x| {
                    x.names
                        .iter()
                        .map(|y| y.to_owned())
                        .collect::<Vec<String>>()
                })
                .flatten()
                .collect::<Vec<String>>(),
        );
    } else {
        candidates = current_option.candidates(&profiles);
    }

    for completion in candidates.iter() {
        if completion.starts_with(args.last().unwrap()) {
            println!("{}", completion);
        }
    }
}

fn main() {
    routine()
}
