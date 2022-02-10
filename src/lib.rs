pub mod structures {

    use serde::de::Visitor;
    use serde::de::{Error, Unexpected};
    use serde::{Deserialize, Deserializer};

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

    pub trait Consumer {
        fn consumables(&self, available: &Vec<&str>, parent: &Command) -> usize;
    }

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    pub enum ArgumentCount {
        Fixed(u64),
        AtMostOne(),
        AtLeastOne(),
        Any(),
    }

    fn int_or_special<'de, D>(deserializer: D) -> Result<ArgumentCount, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArgumentVisitor;

        impl<'de> Visitor<'de> for ArgumentVisitor {
            type Value = ArgumentCount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("enum ArgumentCount")
            }

            fn visit_u64<E>(self, u: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ArgumentCount::Fixed(u))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if s == "ARGS_SOME" {
                    Ok(ArgumentCount::Any())
                } else if s == "ARGS_ATLEASTONE" {
                    Ok(ArgumentCount::AtLeastOne())
                } else if s == "ARGS_ATMOSTONE" {
                    Ok(ArgumentCount::AtMostOne())
                } else {
                    Err(Error::invalid_value(Unexpected::Str(s), &self))
                }
            }
        }

        deserializer.deserialize_any(ArgumentVisitor)
    }

    impl Default for ArgumentCount {
        fn default() -> ArgumentCount {
            return ArgumentCount::Fixed(0);
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Positional {
        #[serde(deserialize_with = "int_or_special")]
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
        #[serde(deserialize_with = "int_or_special")]
        pub arguments: ArgumentCount,

        #[serde(default)]
        pub expected_type: String,
    }

    impl Completable for Option_ {
        fn candidates<'a>(&'a self, _profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            // Complete with possible values
            self.values.iter().map(|x| x.as_str()).collect()
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Command {
        pub name: String,

        #[serde(default)]
        pub subcommands: Vec<Command>,

        #[serde(default)]
        pub positionals: Vec<Positional>,

        #[serde(default)]
        pub options: Vec<Option_>,
    }

    impl Command {
        pub fn is_option(&self, token: &str) -> Option<&Option_> {
            for option in self.options.iter() {
                if option.names.iter().find(|x| x.as_str() == token).is_some() {
                    return Some(option);
                }
            }

            None
        }

        pub fn is_subcommand(&self, token: &str) -> Option<&Command> {
            self.subcommands.iter().find(|c| c.name.as_str() == token)
        }
    }

    impl Completable for Command {
        fn candidates<'a>(&'a self, _profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            // Complete with possible values
            let mut strings: Vec<&str> = vec![];

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
