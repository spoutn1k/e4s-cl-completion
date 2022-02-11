#[macro_use]
extern crate log;

pub mod structures {

    use serde::de::Visitor;
    use serde::de::{Error, Unexpected};
    use serde::{Deserialize, Deserializer};
    use std::collections::HashSet;

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

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    pub enum ArgumentCount {
        Fixed(u64),
        AtMostOne(),
        AtLeastOne(),
        Any(),
    }

    fn argument_count_de<'de, D>(deserializer: D) -> Result<ArgumentCount, D::Error>
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
    #[serde(untagged)]
    pub enum ExpectedType {
        Unknown(),
        Profile(),
        Path(),
    }

    fn expected_type_de<'de, D>(deserializer: D) -> Result<ExpectedType, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TypeVisitor;

        impl<'de> Visitor<'de> for TypeVisitor {
            type Value = ExpectedType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("enum ExpectedType")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match s {
                    "defined_profile" => Ok(ExpectedType::Profile()),
                    _ => Ok(ExpectedType::Unknown()),
                }
            }
        }

        deserializer.deserialize_string(TypeVisitor)
    }

    impl Default for ExpectedType {
        fn default() -> ExpectedType {
            return ExpectedType::Unknown();
        }
    }

    pub trait Completable {
        fn available<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str>;
    }

    #[derive(Deserialize, Debug)]
    pub struct Positional {
        #[serde(default)]
        #[serde(deserialize_with = "argument_count_de")]
        pub arguments: ArgumentCount,

        #[serde(default)]
        #[serde(deserialize_with = "expected_type_de")]
        pub expected_type: ExpectedType,
    }

    impl Completable for Positional {
        fn available<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            debug!("Getting available completion for positional");
            match self.expected_type {
                ExpectedType::Profile() => profiles
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<&str>>(),
                _ => vec![],
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Option_ {
        pub names: Vec<String>,

        #[serde(default)]
        pub values: Vec<String>,

        #[serde(default)]
        #[serde(deserialize_with = "argument_count_de")]
        pub arguments: ArgumentCount,

        #[serde(default)]
        #[serde(deserialize_with = "expected_type_de")]
        pub expected_type: ExpectedType,
    }

    impl Completable for Option_ {
        fn available<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            debug!("Getting available completion for option {:?}", self.names);

            match self.expected_type {
                ExpectedType::Profile() => profiles
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<&str>>(),
                _ => vec![],
            }
        }
    }

    impl Option_ {
        pub fn consume_args<'a, T>(&self, parent: &Command, arguments: &mut T)
        where
            T: Iterator<Item = &'a String>,
        {
            let mut arguments = arguments.peekable();

            match self.arguments {
                ArgumentCount::Fixed(size) => {
                    for _ in 0..size {
                        arguments.next();
                    }
                }

                ArgumentCount::AtMostOne() => {
                    if let Some(value) = arguments.peek() {
                        if parent.is_option(&value).is_some() {
                            arguments.next();
                        }
                    }
                }

                _ => {
                    let mut ended: bool = false;
                    while !ended {
                        if let Some(value) = arguments.peek() {
                            if parent.is_option(&value).is_some() {
                                ended = true;
                            } else {
                                arguments.next();
                            }
                        } else {
                            ended = true;
                        }
                    }
                }
            }
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

    impl Completable for Command {
        fn available<'a>(&'a self, profiles: &'a Vec<Profile>) -> Vec<&'a str> {
            let mut available: Vec<&str>;
            debug!("Getting available completion for command {:?}", self.name);

            available = self
                .options
                .iter()
                .map(|x| x.names.iter().map(|y| y.as_str()).collect::<Vec<&str>>())
                .flatten()
                .collect::<Vec<_>>();

            available.extend(
                self.subcommands
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<&str>>(),
            );

            available.extend(
                self.positionals
                    .iter()
                    .map(|x| x.available(profiles))
                    .flatten()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<&str>>(),
            );

            debug!("Available: {:?}", available);
            available
        }
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

        pub fn candidates<'a>(
            &'a self,
            arguments: &[String],
            profiles: &'a Vec<Profile>,
        ) -> Vec<&'a str> {
            debug!("Completing {:?} with {} command", arguments, self.name);

            let mut iter = arguments.iter().peekable();

            let mut final_object: Option<&Option_> = None;
            let mut positionals_used = 0;
            let mut options_used: Vec<&str> = vec![];

            iter.next();

            while let Some(token) = iter.next() {
                if let Some(option) = self.is_option(token) {
                    final_object = Some(option);
                    /*let index = self
                        .options
                        .iter()
                        .position(|x| x.names == option.names)
                        .unwrap();
                    self.options.remove(index);*/
                    option.consume_args(self, &mut iter);

                    if iter.peek().is_some() {
                        final_object = None;

                        options_used.extend(
                            option
                                .names
                                .iter()
                                .map(|x| x.as_str())
                                .collect::<Vec<&str>>(),
                        );
                    }
                } else {
                    // If we find a token that is not the end and also not
                    // recognized as an option, it has to be a positional
                    if iter.peek().is_some() {
                        debug!("Token {} is not an option, must be a positional", token);
                        positionals_used += 1;
                    }
                }
            }

            if let Some(option) = final_object {
                option.available(profiles)
            } else {
                let mut available: Vec<&str>;
                debug!("Getting available completion for command {:?}", self.name);
                available = self
                    .options
                    .iter()
                    .map(|x| x.names.iter().map(|y| y.as_str()).collect::<Vec<&str>>())
                    .flatten()
                    .collect::<Vec<_>>();

                available.extend(
                    self.subcommands
                        .iter()
                        .map(|x| x.name.as_str())
                        .collect::<Vec<&str>>(),
                );

                // Allow only po
                if positionals_used < self.positionals.len() {
                    available.extend(
                        self.positionals[positionals_used..]
                            .iter()
                            .map(|x| x.available(profiles))
                            .flatten()
                            .collect::<HashSet<_>>()
                            .into_iter()
                            .collect::<Vec<&str>>(),
                    );
                }

                available
            }
        }
    }
}
