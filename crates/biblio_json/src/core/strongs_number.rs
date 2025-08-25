use std::{fmt::Display, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

lazy_static::lazy_static!
{
    static ref STRONGS_REGEX: Regex = Regex::new(r"^(?P<lang>[HG])(?P<number>\d+)$").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrongsLang
{
    Hebrew,
    Greek,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StrongsNumber
{
    pub lang: StrongsLang,
    pub number: u32,
}

impl Display for StrongsNumber
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let letter = match self.lang {
            StrongsLang::Hebrew => "H",
            StrongsLang::Greek => "G",
        };

        write!(f, "{}{}", letter, self.number)
    }
}

impl FromStr for StrongsNumber
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match STRONGS_REGEX.captures(s) 
        {
            Some(captures) => {
                let lang = match captures.name("lang").unwrap().as_str()
                {
                    "H" => StrongsLang::Hebrew,
                    "G" => StrongsLang::Greek,
                    _ => return Err(format!("String {} is not a valid strongs number", s))
                };

                let number = captures.name("number").unwrap().as_str();
                let number = match u32::from_str(number) {
                    Ok(ok) => ok,
                    Err(e) => return Err(e.to_string()),
                };

                Ok(Self {
                    lang,
                    number
                })
            },
            None => Err(format!("String {} is not a valid strongs number", s)),
        }
    }
}

impl Serialize for StrongsNumber 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for StrongsNumber 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        StrongsNumber::from_str(&s).map_err(serde::de::Error::custom)
    }
}
