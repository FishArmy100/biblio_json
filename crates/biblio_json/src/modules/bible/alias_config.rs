use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::{OsisBook, lang::Language};
use maplit::hashmap;



#[derive(Debug, Default)]
pub struct AliasConfig(HashMap<OsisBook, Vec<String>>);

impl Deref for AliasConfig
{
    type Target = HashMap<OsisBook, Vec<String>>;

    fn deref(&self) -> &Self::Target 
    {
        &self.0
    }
}

impl<'de> Deserialize<'de> for AliasConfig 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        // Define an untagged helper enum just for deserialization
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper 
        {
            Str(String),                    // matches when TOML has a string
            Map(HashMap<OsisBook, Vec<String>>), // matches when TOML has a table
        }

        match Helper::deserialize(deserializer)? 
        {
            Helper::Str(s) => {
                let Ok(lang) = Language::new(&s) else {
                    return Err(serde::de::Error::custom(format!("'{}' is not a valid language code", s)))
                };

                let Some(config) = DEFAULT_LANGUAGE_BOOK_ALIASES.get(&lang) else {
                    return Err(serde::de::Error::custom(format!("Language '{}' does not have default aliases", lang)))
                };

                Ok(Self(config.clone()))
            },
            Helper::Map(m) => Ok(AliasConfig(m)),
        }
    }
}

impl Serialize for AliasConfig 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

lazy_static::lazy_static! {
    static ref DEFAULT_LANGUAGE_BOOK_ALIASES: HashMap<Language, HashMap<OsisBook, Vec<String>>> = hashmap! {
        Language::new("en").unwrap() => hashmap! {
            OsisBook::Num => vec!["nm".into()],
            OsisBook::Deut => vec!["dt".into()],
            OsisBook::Josh => vec!["jsh".into()],
            OsisBook::Judg => vec!["jdg".into(), "jdgs".into()],
            OsisBook::Sam1 => vec!["1 sm".into()],
            OsisBook::Sam2 => vec!["2 sm".into()],
            OsisBook::Job => vec!["jb".into()],
            OsisBook::Ps => vec!["pss".into(), "psalms".into()],
            OsisBook::Prov => vec!["prv".into()],
            OsisBook::Song => vec!["sg".into(), "ss".into(), "sos".into()],
            OsisBook::Joel => vec!["jl".into()],
            OsisBook::Obad => vec!["obd".into()],
            OsisBook::Hab => vec!["hb".into()],
            OsisBook::Hag => vec!["hg".into()],
            OsisBook::Mal => vec!["ml".into()],
            OsisBook::Matt => vec!["mt".into()],
            OsisBook::Mark => vec!["mk".into()],
            OsisBook::Luke => vec!["lk".into()],
            OsisBook::John => vec!["jn".into()],
            OsisBook::Jas => vec!["jas".into()],
            OsisBook::Phil => vec!["php".into()],
            OsisBook::Phlm => vec!["phm".into()],
        }
    };
}
