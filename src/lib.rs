pub mod structures {

    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Profile {
        pub name: String,
        /*
        #[serde(default)]
        files: Vec<String>,

        #[serde(default)]
        libraries: Vec<String>,
        */
    }

    pub trait Completable {
        fn candidates<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str>;
    }

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    pub enum ArgumentCount {
        Fixed(i32),
        Special(String),
    }

    impl Default for ArgumentCount {
        fn default() -> ArgumentCount {
            return ArgumentCount::Fixed(0);
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Positional {
        pub arguments: ArgumentCount,
        #[serde(default)]
        pub expected_type: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Option_ {
        pub names: Vec<String>,
        #[serde(default)]
        pub values: Vec<String>,
        #[serde(default)]
        pub arguments: ArgumentCount,
        #[serde(default)]
        pub expected_type: String,
    }

    impl Completable for Option_ {
        fn candidates<'a>(&'a self, _profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            // Complete with possible values
            let strings: Vec<&str> = self.values.iter().map(|x| x.as_str()).collect();

            /* If the values contain the profile keyword, add the profile names
            if self.values.contains(&PROFILE.to_owned()) {
                strings.extend(profiles.iter().map(|x| x.name.as_str()));
            }*/

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
    }

    impl Completable for Command {
        fn candidates<'a>(&'a self, _profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            // Complete with possible values
            let mut strings: Vec<&str> = vec![];

            /* If the values contain the profile keyword, add the profile names
            if self.values.contains(&PROFILE.to_owned()) {
                strings.extend(profiles.iter().map(|x| x.name.as_str()));
            }*/

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

pub mod ex {
    pub static SAMPLE_JSON: &str = include_str!("completion.json");
}
