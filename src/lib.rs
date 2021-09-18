pub mod structures {
    static PATH: &str = "__e4s_cl_path";
    static PROFILE: &str = "__e4s_cl_profile";

    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Profile {
        pub name: String,

        #[serde(default)]
        files: Vec<String>,

        #[serde(default)]
        libraries: Vec<String>,
    }

    pub trait Completable {
        fn candidates<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str>;
    }

    #[derive(Deserialize, Debug)]
    pub struct Option_ {
        pub names: Vec<String>,
        #[serde(default)]
        pub values: Vec<String>,
        #[serde(default)]
        pub arguments: i32,
    }

    impl Completable for Option_ {
        fn candidates<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            // Complete with possible values
            let mut strings: Vec<&str> = self
                .values
                .iter()
                .filter(|x| x.as_str() != PATH && x.as_str() != PROFILE)
                .map(|x| x.as_str())
                .collect();

            // If the values contain the profile keyword, add the profile names
            if self.values.contains(&PROFILE.to_owned()) {
                strings.extend(profiles.iter().map(|x| x.name.as_str()));
            }

            strings
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Command {
        pub name: String,
        #[serde(default)]
        pub subcommands: Vec<Command>,
        #[serde(default)]
        pub options: Vec<Option_>,
        #[serde(default)]
        pub arguments: i32,
        #[serde(default)]
        pub values: Vec<String>,
    }

    impl Completable for Command {
        fn candidates<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            // Complete with possible values
            let mut strings: Vec<&str> = self
                .values
                .iter()
                .filter(|x| x.as_str() != PATH && x.as_str() != PROFILE)
                .map(|x| x.as_str())
                .collect();

            // If the values contain the profile keyword, add the profile names
            if self.values.contains(&PROFILE.to_owned()) {
                strings.extend(profiles.iter().map(|x| x.name.as_str()));
            }

            // Also subcommands
            strings.extend(
                self.subcommands
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<&str>>(),
            );

            // Also options
            strings.extend(
                self.options
                    .iter()
                    .map(|x| x.names.iter().map(|y| y.as_str()).collect::<Vec<&str>>())
                    .flatten()
                    .collect::<Vec<&str>>(),
            );

            strings
        }
    }
}

pub mod init_complete {
    pub static COMMAND: &str = r#"complete -C ./target/debug/__e4s_cl_comp \
    -o bashdefault \
    -o default \
    -o filenames \
    e4s-cl"#;
}

pub mod ex {
    pub static SAMPLE_JSON: &str = r#"
{"name":"root","subcommands":[{"name":"__analyze","options":[{"names":["-h","--help"]},{"names":["--libraries"],"arguments":1}]},{"name":"__execute","options":[{"names":["-h","--help"]},{"names":["--backend"],"arguments":1},{"names":["--image"],"arguments":1},{"names":["--files"],"arguments":1},{"names":["--libraries"],"arguments":1},{"names":["--source"],"arguments":1}]},{"name":"help","options":[{"names":["-h","--help"]}]},{"name":"init","options":[{"names":["-h","--help"]},{"names":["--launcher"],"arguments":1},{"names":["--mpi"],"arguments":1},{"names":["--source"],"arguments":1},{"names":["--image"],"arguments":1},{"names":["--backend"],"arguments":1},{"names":["--profile"],"arguments":1,"values":["__e4s_cl_profile"]},{"names":["--wi4mpi"],"arguments":1},{"names":["--wi4mpi_options"],"arguments":1}]},{"name":"launch","options":[{"names":["-h","--help"]},{"names":["--profile"],"arguments":1,"values":["__e4s_cl_profile"]},{"names":["--image"],"arguments":1},{"names":["--source"],"arguments":1},{"names":["--files"],"arguments":1},{"names":["--libraries"],"arguments":1},{"names":["--backend"],"arguments":1}]},{"name":"profile","subcommands":[{"name":"copy","options":[{"names":["-h","--help"]},{"names":["-@"],"arguments":1,"values":["user","system"]}],"arguments":1,"values":["__e4s_cl_profile"]},{"name":"create","options":[{"names":["-h","--help"]},{"names":["--libraries"],"arguments":1},{"names":["--files"],"arguments":1},{"names":["--backend"],"arguments":1},{"names":["--image"],"arguments":1},{"names":["--source"],"arguments":1},{"names":["--wi4mpi"],"arguments":1},{"names":["--wi4mpi_options"],"arguments":1}]},{"name":"delete","options":[{"names":["-h","--help"]},{"names":["-@"],"arguments":1,"values":["user","system"]}]},{"name":"detect","options":[{"names":["-h","--help"]},{"names":["-p","--profile"],"arguments":1}]},{"name":"diff","options":[{"names":["-h","--help"]}],"arguments":1,"values":["__e4s_cl_profile"]},{"name":"dump","options":[{"names":["-h","--help"]},{"names":["-@"],"arguments":1,"values":["user","system"]}]},{"name":"edit","options":[{"names":["-h","--help"]},{"names":["--new_name"],"arguments":1},{"names":["--backend"],"arguments":1},{"names":["--image"],"arguments":1},{"names":["--source"],"arguments":1},{"names":["--add-files"],"arguments":1},{"names":["--remove-files"],"arguments":1},{"names":["--add-libraries"],"arguments":1},{"names":["--remove-libraries"],"arguments":1},{"names":["--wi4mpi"],"arguments":1},{"names":["--wi4mpi_options"],"arguments":1}],"arguments":1,"values":["__e4s_cl_profile"]},{"name":"list","options":[{"names":["-h","--help"]},{"names":["-s","--short"]},{"names":["-d","--dashboard"]},{"names":["-l","--long"]},{"names":["-@"],"arguments":1,"values":["user","system"]}]},{"name":"select","options":[{"names":["-h","--help"]}],"arguments":1,"values":["__e4s_cl_profile"]},{"name":"show","options":[{"names":["-h","--help"]},{"names":["--tree"]}],"arguments":1,"values":["__e4s_cl_profile"]},{"name":"unselect","options":[{"names":["-h","--help"]}],"arguments":1,"values":["__e4s_cl_profile"]}],"options":[{"names":["-h","--help"]}]}],"options":[{"names":["-h","--help"]},{"names":["-V","--version"]},{"names":["-v","--verbose"]},{"names":["-q","--quiet"]},{"names":["-d","--dry-run"]},{"names":["--slave"]}]}
"#;
}
