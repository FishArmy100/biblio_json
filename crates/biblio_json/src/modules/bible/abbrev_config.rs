use std::{collections::HashMap, ops::Deref};

use maplit::hashmap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::{OsisBook, lang::Language};




#[derive(Debug, Default)]
pub struct AbbrevConfig(HashMap<OsisBook, String>);

impl Deref for AbbrevConfig
{
    type Target = HashMap<OsisBook, String>;

    fn deref(&self) -> &Self::Target 
    {
        &self.0
    }
}

impl<'de> Deserialize<'de> for AbbrevConfig 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper 
        {
            Str(String),
            Map(HashMap<OsisBook, String>),
        }

        match Helper::deserialize(deserializer)? 
        {
            Helper::Str(s) => {
                let Ok(lang) = Language::new(&s) else {
                    return Err(serde::de::Error::custom(format!("'{}' is not a valid language code", s)))
                };

                let Some(config) = DEFAULT_LANGUAGE_BOOK_ABBREVIATIONS.get(&lang) else {
                    return Err(serde::de::Error::custom(format!("Language '{}' does not have default abbreviations", lang)))
                };

                Ok(Self(config.clone()))
            },
            Helper::Map(m) => Ok(Self(m)),
        }
    }
}

impl Serialize for AbbrevConfig 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        self.0.serialize(serializer)
    }
}


lazy_static::lazy_static! {
    static ref DEFAULT_LANGUAGE_BOOK_ABBREVIATIONS: HashMap<Language, HashMap<OsisBook, String>> = hashmap! {
        Language::new("en").unwrap() => hashmap! {
            OsisBook::Gen   => "Gen".to_string(),
            OsisBook::Exod  => "Exod".to_string(),
            OsisBook::Lev   => "Lev".to_string(),
            OsisBook::Num   => "Num".to_string(),
            OsisBook::Deut  => "Deut".to_string(),
            OsisBook::Josh  => "Josh".to_string(),
            OsisBook::Judg  => "Judg".to_string(),
            OsisBook::Sam1  => "1 Sam".to_string(),
            OsisBook::Sam2  => "2 Sam".to_string(),
            OsisBook::Kgs1  => "1 Kgs".to_string(),
            OsisBook::Kgs2  => "2 Kgs".to_string(),
            OsisBook::Chr1  => "1 Chr".to_string(),
            OsisBook::Chr2  => "2 Chr".to_string(),
            OsisBook::Neh => "Neh".to_string(),
            OsisBook::Esth  => "Esth".to_string(),
            OsisBook::Ps    => "Ps".to_string(),
            OsisBook::Prov  => "Prov".to_string(),
            OsisBook::Eccl  => "Eccl".to_string(),
            OsisBook::Song  => "Song".to_string(),
            OsisBook::Isa   => "Isa".to_string(),
            OsisBook::Jer   => "Jer".to_string(),
            OsisBook::Lam   => "Lam".to_string(),
            OsisBook::Ezek  => "Ezek".to_string(),
            OsisBook::Dan   => "Dan".to_string(),
            OsisBook::Hos   => "Hos".to_string(),
            OsisBook::Amos  => "Amos".to_string(),
            OsisBook::Obad  => "Obad".to_string(),
            OsisBook::Mic   => "Mic".to_string(),
            OsisBook::Nah   => "Nah".to_string(),
            OsisBook::Hab   => "Hab".to_string(),
            OsisBook::Zeph  => "Zeph".to_string(),
            OsisBook::Hag   => "Hag".to_string(),
            OsisBook::Zech  => "Zech".to_string(),
            OsisBook::Mal   => "Mal".to_string(),
            OsisBook::Matt  => "Matt".to_string(),
            OsisBook::John => "Jn".to_string(),
            OsisBook::Rom   => "Rom".to_string(),
            OsisBook::Cor1  => "1 Cor".to_string(),
            OsisBook::Cor2  => "2 Cor".to_string(),
            OsisBook::Gal   => "Gal".to_string(),
            OsisBook::Eph   => "Eph".to_string(),
            OsisBook::Phil  => "Phil".to_string(),
            OsisBook::Col   => "Col".to_string(),
            OsisBook::Thess1 => "1 Thess".to_string(),
            OsisBook::Thess2 => "2 Thess".to_string(),
            OsisBook::Tim1  => "1 Tim".to_string(),
            OsisBook::Tim2  => "2 Tim".to_string(),
            OsisBook::Phlm  => "Phlm".to_string(),
            OsisBook::Heb   => "Heb".to_string(),
            OsisBook::Jas   => "Jas".to_string(),
            OsisBook::Pet1  => "1 Pet".to_string(),
            OsisBook::Pet2  => "2 Pet".to_string(),
            OsisBook::John1 => "1 Jn".to_string(),
            OsisBook::John2 => "2 Jn".to_string(),
            OsisBook::John3 => "3 Jn".to_string(),
            OsisBook::Rev   => "Rev".to_string(),
        }
    };
}
