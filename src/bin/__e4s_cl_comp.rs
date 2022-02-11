#[macro_use]
extern crate log;

use dirs::home_dir;
use e4s_cl_completion::structures::{Command, Profile};
use shlex::split;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

static ENV_LINE_VAR: &str = "COMP_LINE";
static DATABASE: &'static str = ".local/e4s_cl/user.json";

#[derive(Debug)]
struct DeserializationError(String);

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for DeserializationError {}

fn load_profiles<P: AsRef<Path>>(path: P) -> Result<Vec<Profile>, Box<dyn Error>> {
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
        None => Err(Box::new(DeserializationError(
            "Deserialization failed".to_string(),
        ))),
    }
}

fn load_commands() -> Result<Command, Box<dyn Error>> {
    Ok(serde_json::from_str(include_str!("completion.json"))?)
}

fn context_end(command: &Command, arguments: &[String]) -> usize {
    let mut iter = arguments.iter();

    while let Some(value) = iter.next() {
        if let Some(option) = command.is_option(value) {
            option.consume_args(command, &mut iter);
        }

        if let Some(_) = command.is_subcommand(value) {
            debug!("{} is a subcommand of {} !", value, command.name);
            break;
        }
    }

    let mut remaining: usize = 0;
    while let Some(_) = iter.next() {
        remaining += 1;
    }

    if remaining > 0 {
        let context_end = arguments.len() - remaining - 1;

        debug!(
            "{} context is {:?}, switches for {:?}",
            command.name,
            &arguments[..context_end],
            &arguments[context_end..],
        );

        context_end
    } else {
        debug!(
            "All the arguments in {:?} belong to the {} context",
            arguments, command.name
        );
        0
    }
}

fn routine(arguments: &Vec<String>) {
    let root_command: Command;

    let db_file = home_dir().unwrap().join(DATABASE);

    let candidates: Vec<&str>;
    let profiles: Vec<Profile>;

    match load_commands() {
        Ok(object) => root_command = object,
        Err(error) => {
            error!("Error loading JSON: {}", error);
            return;
        }
    }

    debug!("Loaded commands");

    match load_profiles(db_file) {
        Ok(data) => profiles = data,
        Err(error) => {
            error!("Loading profiles failed: {:?}", error);
            profiles = vec![];
        }
    }

    debug!("Loaded profiles: {:?}", profiles);

    let mut pos = 0;
    let mut context_path: Vec<&Command> = vec![&root_command];

    while pos < arguments.len() {
        let token = &arguments[pos];
        let context = context_path.last().unwrap();

        debug!(
            "Context is command '{}' ({})",
            context.name,
            context_path
                .iter()
                .map(|c| c.name.to_owned())
                .collect::<Vec<String>>()
                .join(" => ")
        );

        if token.len() == 0 {
            pos += 1;
            continue;
        }

        let skip = context_end(context, &arguments[pos..]);

        if skip > 0 {
            pos += skip;

            if let Some(command) = context.is_subcommand(&arguments[pos]) {
                context_path.insert(context_path.len(), command);
            } else {
                // Raise an error here, misunderstood command line
            }
        } else {
            break;
        }
    }

    candidates = context_path
        .last()
        .unwrap()
        .candidates(&arguments[pos..], &profiles);

    debug!("Candidates: {:?}", candidates);

    for completion in candidates.iter() {
        if completion.starts_with(arguments.last().unwrap()) {
            println!("{}", completion);
        }
    }
}

fn main() {
    let mut args = env::args();
    let command_line: Vec<String>;

    if cfg!(debug_assertions) {
        WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            File::create("/tmp/e4s-cl-completion").unwrap(),
        )
        .unwrap();
        debug!("Initialized logging #################################");
    }

    // Get the completion line from the environment
    let raw_cli = std::env::var(&ENV_LINE_VAR);
    if raw_cli.is_err() {
        println!(include_str!("complete.fmt"), args.next().unwrap());
        exit(0);
    }

    let string = raw_cli.unwrap();

    // Chop it into parts, to understand what has already been written
    match split(&string) {
        Some(mut data) => {
            // Add a final element if finished by a space
            if string.chars().last().unwrap() == ' ' {
                data.insert(data.len(), "".to_string())
            }

            debug!("Command line: {:?}", data);
            command_line = data;
        }
        None => {
            error!("Command line split failed !");
            exit(0);
        }
    }

    routine(&command_line)
}
