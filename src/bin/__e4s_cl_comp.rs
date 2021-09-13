use dirs::home_dir;
use serde_json::json;
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

static UNSPECIFIED: i64 = -1;
static PATH: i64 = 10;
static PROFILE: i64 = 20;

static DATABASE: &'static str = ".local/e4s_cl/user.json";
//static DATABASE: &'static Path = Path::new(".local/e4s_cl/user.json");

fn path(stub: &str) -> Vec<String> {
    let current_word_parts: Vec<&str> = stub.rsplitn(2, "/").collect();

    let prefix: &str;

    if stub.starts_with("/") {
        prefix = "/";
    } else {
        prefix = "./";
    }

    let (root_path, partial_path) = match current_word_parts.len() {
        2 => {
            if current_word_parts[1] == "" {
                (prefix, current_word_parts[1])
            } else {
                (current_word_parts[1], current_word_parts[0])
            }
        }
        1 => (prefix, current_word_parts[0]),
        0 => (prefix, ""),
        _ => unreachable!(),
    };

    struct El {
        dir: bool,
        path: String,
    }

    match std::fs::read_dir(&root_path) {
        Ok(iter) => {
            let paths = iter
                .filter_map(|r| r.ok())
                .map(|el| El {
                    dir: el.metadata().unwrap().is_dir(),
                    path: el.path().to_string_lossy().into_owned(),
                })
                .filter(|el| {
                    el.path
                        .rsplitn(2, "/")
                        .next()
                        .unwrap()
                        .starts_with(partial_path)
                })
                .map(|el| {
                    eprintln!("\n--- {} {} ---\n", el.path, el.dir);
                    let mut path: String = el.path.into();
                    if el.dir {
                        path.push_str("/")
                    }

                    path
                });

            if stub.starts_with("./") {
                paths.collect()
            } else {
                paths
                    .map(|p| p.trim_start_matches("./").to_string())
                    .collect()
            }
        }

        Err(_) => vec![],
    }
}

fn subcommands(desc: &serde_json::Value) -> Vec<&str> {
    match desc["subcommands"].as_array() {
        Some(subs) => subs.iter().map(|x| x["name"].as_str().unwrap()).collect(),
        None => vec![],
    }
}

fn get_subcommand<'a>(desc: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
    match desc["subcommands"].as_array() {
        Some(commands) => {
            for command in commands.iter() {
                if command["name"].as_str().unwrap() == name {
                    return Some(command);
                }
            }

            None
        }
        None => None,
    }
}

fn options(desc: &serde_json::Value) -> Vec<&str> {
    match desc["options"].as_array() {
        Some(subs) => subs
            .iter()
            .map(|x| x["names"].as_array().unwrap())
            .flatten()
            .map(|x| x.as_str().unwrap())
            .collect(),
        None => vec![],
    }
}

fn get_option<'a>(desc: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
    match desc["options"].as_array() {
        Some(options) => {
            for option in options.iter() {
                if option["names"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x.as_str().unwrap() == name)
                    .is_some()
                {
                    return Some(option);
                }
            }

            None
        }
        None => None,
    }
}

fn get_argument_candidates<'a>(
    desc: &'a serde_json::Value,
    profiles: &'a Vec<serde_json::Value>,
    stub: &str,
) -> Vec<String> {
    if !desc["values"].is_array() {
        return vec![];
    }

    let data = desc["values"].as_array().unwrap();

    let mut strings: Vec<String> = data
        .iter()
        .filter(|x| x.is_string())
        .map(|x| x.to_string())
        .collect();

    let modifiers: Vec<i64> = data
        .iter()
        .filter(|x| x.is_number())
        .map(|x| x.as_i64().unwrap())
        .collect();

    if modifiers.contains(&PROFILE) {
        strings.extend(profiles.iter().map(|x| x["name"].to_string()));
    }

    if modifiers.contains(&PATH) {
        strings.extend(path(stub).iter().map(|x| x.to_string()));
    }

    strings
}

fn read_profiles<P: AsRef<Path>>(path: P) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file.
    let data: serde_json::Value = serde_json::from_reader(reader)?;

    match data["Profile"].as_object() {
        Some(map) => Ok(map
            .iter()
            .map(|(_i, data)| serde_json::to_value(data).unwrap())
            .collect()),
        None => Ok(vec![]),
    }
}

fn main() {
    let root_command: serde_json::Value = json!({
        "name": "root",
        "options": [{
                "names": ["-v", "--version"],
                "arguments": 0
            },
            {
                "names": ["-d", "--debug"],
                "arguments": 0
            }
        ],
        "subcommands": [{
            "name": "launch",
            "options": [{
                    "names": ["--image"],
                    "values": [PATH],
                    "arguments": 1
                },
                {
                    "names": ["--profile"],
                    "values": [PROFILE],
                    "arguments": 1
                },
                {
                    "names": ["--backend"],
                    "values": ["singularity"],
                    "arguments": 1
                }
            ]
        }, {
            "name": "profile",
            "subcommands": [{
                "name": "list",
                "values": [PROFILE],
                "arguments": 1,
                "options": [{
                    "names": ["-s", "--short"]
                }]
            }, {
                "name": "show",
                "values": [PROFILE],
                "arguments": 1
            }]
        }]
    });

    let env_line_var: &str = "COMP_LINE";
    let args: Vec<String>;

    let mut candidates: Vec<String>;

    let db_file = home_dir().unwrap().join(DATABASE);
    let profiles: Vec<serde_json::Value> = read_profiles(db_file).unwrap_or(vec![]);

    match std::env::var(&env_line_var) {
        Ok(string) => args = string.split(" ").map(|s| s.to_string()).collect(),
        Err(string) => {
            eprintln!("Error accessing {}: {}", env_line_var, string);
            exit(1);
        }
    }

    let empty_json = json!(null);

    let mut pos = 1;
    let mut current_command: &serde_json::Value = &root_command;
    let mut current_option: &serde_json::Value = &empty_json;

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
                current_option = &empty_json;
                pos += 1;
                continue;
            }
            None => (),
        }

        match get_option(&current_command, &token) {
            // Get the option if it exists
            Some(option) => match option["arguments"].as_i64() {
                // Act depending on the number of expected arguments
                Some(n_args) => {
                    if n_args != UNSPECIFIED {
                        // n_args > 0
                        let n_args = usize::try_from(n_args).unwrap();
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
            },
            None => (),
        }

        pos += 1;
    }

    if current_option.is_null() {
        candidates = get_argument_candidates(&current_command, &profiles, args.last().unwrap());
        candidates.extend(subcommands(&current_command).iter().map(|x| x.to_string()));
        candidates.extend(options(&current_command).iter().map(|x| x.to_string()));
    } else {
        candidates = get_argument_candidates(&current_option, &profiles, args.last().unwrap());
    }

    for completion in candidates.iter() {
        if completion.starts_with(args.last().unwrap()) {
            println!("{}", completion);
        }
    }
}
