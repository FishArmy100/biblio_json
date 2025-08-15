use std::{collections::HashMap, fmt::Display, str::FromStr};

use bimap::BiMap;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{ref_id::RefId, utils};

lazy_static::lazy_static!
{
    static ref WORD_RANGE_REGEX: Regex = Regex::new("(?P<start>\\d+)-(?P<end>\\d+)").unwrap();
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LinkerConfig
{
    pub name: String,
    pub bible: String,
    pub ref_works: BiMap<String, u32>,
}

#[derive(Debug)]
pub struct LinkerModule
{
    pub config: LinkerConfig,
    pub links: HashMap<RefId, VerseLink>
}

impl LinkerModule
{
    pub fn load(dir_path: &str, name: &str) -> Result<LinkerModule, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: LinkerConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let links = VerseLink::from_file(&bible_path)?;

        Ok(Self {
            config,
            links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerseLink
{
    pub ref_id: RefId,
    pub links: HashMap<WordRange, Vec<LinkRef>>
}

impl VerseLink
{
    pub fn from_file(path: &str) -> Result<HashMap<RefId, Self>, String>
    {
        utils::load_json_lines::<Self, &str>(path)?
            .into_iter()
            .map(|(l, _)| -> Result<(RefId, VerseLink), String> {
                if let Err(err) = l.validate() {
                    return Err(err);
                }

                Ok((l.ref_id.clone(), l))
            })
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> 
    {
        if self.ref_id.is_range() || !self.ref_id.is_verse()
        {
            return Err(format!("RefId {} is not in format Book.1.1", self.ref_id))
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinkRef
{
    pub ref_work_id: u32,
    pub work_index: u32,
}

/// Word range inclusive
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WordRange
{
    pub start: u32,
    pub end: u32,
}

impl Display for WordRange
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{}:{}", self.start, self.end)
    }
}

impl FromStr for WordRange
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match WORD_RANGE_REGEX.captures(s)
        {
            Some(captures) => {
                let start = captures.name("start").unwrap().as_str();
                let start = match u32::from_str(start) {
                    Ok(ok) => ok,
                    Err(e) => return Err(e.to_string())
                };

                let end = captures.name("end").unwrap().as_str();
                let end = match u32::from_str(end) {
                    Ok(ok) => ok,
                    Err(e) => return Err(e.to_string()),
                };

                Ok(Self {
                    start,
                    end,
                })
            },
            None => Err(format!("String {} is not a valid word range", s)),
        }
    }
}

impl Serialize for WordRange 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WordRange 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let s = String::deserialize(deserializer)?;
        WordRange::from_str(&s).map_err(serde::de::Error::custom)
    }
}