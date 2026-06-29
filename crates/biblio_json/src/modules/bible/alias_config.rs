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
            HashMap::from([
                ("nm".to_string(), OsisBook::Num),
                ("nb".to_string(), OsisBook::Num),

                ("dt".to_string(), OsisBook::Deut),

                ("jsh".to_string(), OsisBook::Josh),
                ("jos".to_string(), OsisBook::Josh),

                ("jdg".to_string(), OsisBook::Judg),
                ("jdgs".to_string(), OsisBook::Judg),
                ("jg".to_string(), OsisBook::Judg),

                ("1 sm".to_string(), OsisBook::Sam1),
                ("2 sm".to_string(), OsisBook::Sam2),

                ("1 kgs".to_string(), OsisBook::Kgs1),
                ("2 kgs".to_string(), OsisBook::Kgs2),

                ("jb".to_string(), OsisBook::Job),

                ("pss".to_string(), OsisBook::Ps),
                ("psm".to_string(), OsisBook::Ps),
                ("psalms".to_string(), OsisBook::Ps),

                ("prv".to_string(), OsisBook::Prov),

                ("sg".to_string(), OsisBook::Song),
                ("ss".to_string(), OsisBook::Song),
                ("sos".to_string(), OsisBook::Song),
                ("song".to_string(), OsisBook::Song),
                ("cant".to_string(), OsisBook::Song),
                ("canticles".to_string(), OsisBook::Song),
                ("song of songs".to_string(), OsisBook::Song),

                ("ecc".to_string(), OsisBook::Eccl),
                ("eccl".to_string(), OsisBook::Eccl),
                ("qoh".to_string(), OsisBook::Eccl),

                ("jl".to_string(), OsisBook::Joel),

                ("ob".to_string(), OsisBook::Obad),
                ("oba".to_string(), OsisBook::Obad),
                ("obd".to_string(), OsisBook::Obad),

                ("hb".to_string(), OsisBook::Hab),
                ("hab".to_string(), OsisBook::Hab),

                ("zep".to_string(), OsisBook::Zeph),

                ("hg".to_string(), OsisBook::Hag),

                ("zec".to_string(), OsisBook::Zech),

                ("ml".to_string(), OsisBook::Mal),

                ("mt".to_string(), OsisBook::Matt),

                ("mr".to_string(), OsisBook::Mark),
                ("mk".to_string(), OsisBook::Mark),

                ("lu".to_string(), OsisBook::Luke),
                ("lk".to_string(), OsisBook::Luke),

                ("jhn".to_string(), OsisBook::John),
                ("jn".to_string(), OsisBook::John),

                ("1 jn".to_string(), OsisBook::John1),
                ("2 jn".to_string(), OsisBook::John2),
                ("3 jn".to_string(), OsisBook::John3),

                ("jam".to_string(), OsisBook::Jas),
                ("jas".to_string(), OsisBook::Jas),

                ("php".to_string(), OsisBook::Phil),

                ("phm".to_string(), OsisBook::Phlm),

                ("jud".to_string(), OsisBook::Jude),

                ("re".to_string(), OsisBook::Rev),
                ("apoc".to_string(), OsisBook::Rev),
            ])
        },

        Language::new("sw").unwrap() => {
            HashMap::from([
                ("mwa".to_string(), OsisBook::Matt),
                ("mk".to_string(), OsisBook::Mark),
                ("lk".to_string(), OsisBook::Luke),
                ("yn".to_string(), OsisBook::John),
                ("mdo".to_string(), OsisBook::Acts),
                ("rum".to_string(), OsisBook::Rom),
                ("uf".to_string(), OsisBook::Eph),
                ("flp".to_string(), OsisBook::Phil),
                ("kol".to_string(), OsisBook::Col),
                ("ebr".to_string(), OsisBook::Heb),
                ("yak".to_string(), OsisBook::Jas),
                ("ufu".to_string(), OsisBook::Rev),
            ])
        },

        Language::new("es").unwrap() => {
            HashMap::from([
                ("gn".to_string(), OsisBook::Gen),
                ("ex".to_string(), OsisBook::Exod),
                ("lv".to_string(), OsisBook::Lev),
                ("nm".to_string(), OsisBook::Num),
                ("dt".to_string(), OsisBook::Deut),
                ("jos".to_string(), OsisBook::Josh),
                ("jue".to_string(), OsisBook::Judg),
                ("1 sam".to_string(), OsisBook::Sam1),
                ("2 sam".to_string(), OsisBook::Sam2),
                ("1 rey".to_string(), OsisBook::Kgs1),
                ("2 rey".to_string(), OsisBook::Kgs2),
                ("sal".to_string(), OsisBook::Ps),
                ("prov".to_string(), OsisBook::Prov),
                ("cant".to_string(), OsisBook::Song),
                ("ecl".to_string(), OsisBook::Eccl),
                ("is".to_string(), OsisBook::Isa),
                ("jer".to_string(), OsisBook::Jer),
                ("ez".to_string(), OsisBook::Ezek),
                ("dn".to_string(), OsisBook::Dan),
                ("os".to_string(), OsisBook::Hos),
                ("jl".to_string(), OsisBook::Joel),
                ("abd".to_string(), OsisBook::Obad),
                ("hab".to_string(), OsisBook::Hab),
                ("hag".to_string(), OsisBook::Hag),
                ("zac".to_string(), OsisBook::Zech),
                ("mal".to_string(), OsisBook::Mal),
                ("mt".to_string(), OsisBook::Matt),
                ("mc".to_string(), OsisBook::Mark),
                ("lc".to_string(), OsisBook::Luke),
                ("jn".to_string(), OsisBook::John),
                ("hch".to_string(), OsisBook::Acts),
                ("rom".to_string(), OsisBook::Rom),
                ("1 cor".to_string(), OsisBook::Cor1),
                ("2 cor".to_string(), OsisBook::Cor2),
                ("gal".to_string(), OsisBook::Gal),
                ("ef".to_string(), OsisBook::Eph),
                ("fil".to_string(), OsisBook::Phil),
                ("col".to_string(), OsisBook::Col),
                ("heb".to_string(), OsisBook::Heb),
                ("stg".to_string(), OsisBook::Jas),
                ("ap".to_string(), OsisBook::Rev),
            ])
        }
    };
}
