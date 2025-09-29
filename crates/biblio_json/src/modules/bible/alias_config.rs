use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::{OsisBook, lang::Language};
use maplit::hashmap;

#[derive(Debug, Default)]
pub struct AliasConfig(HashMap<String, OsisBook>);

impl Deref for AliasConfig
{
    type Target = HashMap<String, OsisBook>;

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
            Map(HashMap<String, OsisBook>), // matches when TOML has a table
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
    static ref DEFAULT_LANGUAGE_BOOK_ALIASES: HashMap<Language, HashMap<String, OsisBook>> = hashmap! {
        Language::new("en").unwrap() => {
            let map = HashMap::from([
                ("nm".to_string(), OsisBook::Num),
                ("dt".to_string(), OsisBook::Deut),
                ("jsh".to_string(), OsisBook::Josh),
                ("jdg".to_string(), OsisBook::Judg),
                ("jdgs".to_string(), OsisBook::Judg),
                ("1 sm".to_string(), OsisBook::Sam1),
                ("2 sm".to_string(), OsisBook::Sam2),
                ("jb".to_string(), OsisBook::Job),
                ("pss".to_string(), OsisBook::Ps),
                ("psalms".to_string(), OsisBook::Ps),
                ("prv".to_string(), OsisBook::Prov),
                ("sg".to_string(), OsisBook::Song),
                ("ss".to_string(), OsisBook::Song),
                ("sos".to_string(), OsisBook::Song),
                ("jl".to_string(), OsisBook::Joel),
                ("obd".to_string(), OsisBook::Obad),
                ("hb".to_string(), OsisBook::Hab),
                ("hg".to_string(), OsisBook::Hag),
                ("ml".to_string(), OsisBook::Mal),
                ("mt".to_string(), OsisBook::Matt),
                ("mk".to_string(), OsisBook::Mark),
                ("lk".to_string(), OsisBook::Luke),
                ("jn".to_string(), OsisBook::John),
                ("1 jn".to_string(), OsisBook::John1),
                ("2 jn".to_string(), OsisBook::John2),
                ("3 jn".to_string(), OsisBook::John3),
                ("jas".to_string(), OsisBook::Jas),
                ("php".to_string(), OsisBook::Phil),
                ("phm".to_string(), OsisBook::Phlm),
            ]);
            map
        }
    };
}
