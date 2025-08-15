use std::{collections::HashMap, fmt::Display, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::utils;

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

                let number = captures.name("end").unwrap().as_str();
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrongsEntry
{
    pub strongs_ref: StrongsNumber,
    pub word: String,
    pub definitions: Vec<String>,
    pub derivation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrongsDefsConfig
{
    pub name: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug)]
pub struct StrongsDefsModule
{
    pub config: StrongsDefsConfig,
    pub index_map: HashMap<StrongsNumber, u32>,
    pub defs: Vec<StrongsEntry>,
}

impl StrongsDefsModule
{
    pub fn load(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: StrongsDefsConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let defs = StrongsEntry::from_file(&bible_path)?;

        let index_map: HashMap<_, _> = defs.iter().enumerate().map(|(i, d)| {
            (d.strongs_ref.clone(), i as u32)
        }).collect();

        Ok(Self { 
            config,
            index_map,
            defs,
        })
    }

    pub fn get_def(&self, num: &StrongsNumber) -> Option<&StrongsEntry>
    {
        match self.index_map.get(num)
        {
            Some(idx) => Some(&self.defs[*idx as usize]),
            None => None
        }
    }
}

impl StrongsEntry
{
    pub fn from_file(path: &str) -> Result<Vec<Self>, String>
    {
        let ret = utils::load_json_lines(path)?
            .into_iter()
            .map(|(l, _)| l)
            .collect();

        Ok(ret)
    }
}