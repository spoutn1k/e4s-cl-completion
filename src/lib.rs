pub mod structures {
    static PATH: &str = "__e4s_cl_path";
    static PROFILE: &str = "__e4s_cl_profile";

    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Profile {
        pub name: String,

        #[serde(default)]
        files: Vec<String>,

        #[serde(default)]
        libraries: Vec<String>,
        /*
        #[serde(default)]
        image: Option<String>,

        #[serde(default)]
        backend: Option<String>,

        #[serde(default)]
        source: Option<String>,

        #[serde(default)]
        wi4mpi: Option<String>,

        #[serde(default)]
         wi4mpi_options: Option<String>,
         */
    }

    pub trait Completable {
        fn candidates(&self, profiles: &Vec<Profile>) -> Vec<String>;
    }

    #[derive(Deserialize)]
    pub struct Option_ {
        pub names: Vec<String>,
        pub values: Vec<String>,
        pub arguments: i32,
    }

    impl Completable for Option_ {
        fn candidates(&self, profiles: &Vec<Profile>) -> Vec<String> {
            let mut strings: Vec<String> = self
                .values
                .iter()
                .filter(|x| x.as_str() != PATH && x.as_str() != PROFILE)
                .map(|x| x.to_owned())
                .collect();

            if self.values.contains(&PROFILE.to_owned()) {
                strings.extend(profiles.iter().map(|x| x.name.to_owned()));
            }

            strings
        }
    }

    #[derive(Deserialize)]
    pub struct Command {
        pub name: String,
        pub subcommands: Vec<Command>,
        pub options: Vec<Option_>,
        pub arguments: i32,
        pub values: Vec<String>,
    }

    impl Completable for Command {
        fn candidates(&self, profiles: &Vec<Profile>) -> Vec<String> {
            let mut strings: Vec<String> = self
                .values
                .iter()
                .filter(|x| x.as_str() != PATH && x.as_str() != PROFILE)
                .map(|x| x.to_owned())
                .collect();

            if self.values.contains(&PROFILE.to_owned()) {
                strings.extend(profiles.iter().map(|x| x.name.to_owned()));
            }

            strings
        }
    }
}

pub mod ex {
    pub static SAMPLE_JSON: &str = r#"
{
    "name": "root",
    "options": [{
        "names": ["-v", "--version"],
        "arguments": 0
    }, {
        "names": ["-d", "--debug"],
        "arguments": 0
    }],
    "subcommands": [{
        "name": "launch",
        "options": [{
            "names": ["--image"],
            "values": [10],
            "arguments": 1
        }, {
            "names": ["--profile"],
            "values": [20],
            "arguments": 1
        }, {
            "names": ["--backend"],
            "values": ["singularity"],
            "arguments": 1
        }]
    }, {
        "name": "profile",
        "subcommands": [{
            "name": "list",
            "values": [20],
            "arguments": 1,
            "options": [{
                "names": ["-s", "--short"]
            }]
        }, {
            "name": "show",
            "values": [20],
            "arguments": 1
        }]
    }]
}
"#;
}
