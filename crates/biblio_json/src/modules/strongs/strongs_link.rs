use std::{collections::HashMap, fmt::Display, num::NonZeroU32, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{core::{RefId, StrongsNumber, VerseId, lang::Language}, html_text::HtmlText, modules::{ExternalModuleData, ModuleValidationContext, ModuleValidationError}, utils};

lazy_static::lazy_static!
{
    static ref WORD_RANGE_REGEX: Regex = Regex::new("^(?P<start>[1-9]\\d*)-(?P<end>[1-9]\\d*)$").unwrap();
    static ref WORD_INDEX_REGEX: Regex = Regex::new("^(?P<index>[1-9]\\d*)$").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrongsLinkConfig
{
    pub name: String,
    pub bible: String,
    pub authors: Option<Vec<String>>,
    pub language: Option<Language>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub license: Option<String>,
    #[serde(default)]
    pub external: ExternalModuleData,
}

#[derive(Debug)]
pub struct StrongsLinksModule
{
    pub config: StrongsLinkConfig,
    pub index_map: HashMap<VerseId, u32>,
    pub links: Vec<StrongsLinkEntry>,
}

impl StrongsLinksModule
{
    pub fn load(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: StrongsLinkConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let links = StrongsLinkEntry::from_file(&bible_path)?;

        let index_map: HashMap<_, _> = links.iter().enumerate().map(|(i, d)| {
            (d.verse_id.clone(), i as u32)
        }).collect();

        Ok(Self { 
            config,
            index_map,
            links,
        })
    }

    pub fn get_def(&self, verse: &VerseId) -> Option<&StrongsLinkEntry>
    {
        match self.index_map.get(verse)
        {
            Some(idx) => Some(&self.links[*idx as usize]),
            None => None
        }
    }

    pub fn validate(&self, context: &ModuleValidationContext) -> Result<(), Vec<ModuleValidationError>>
    {
        let bible_name = &self.config.bible;
        let Some(bible) = context.bibles.get(bible_name) else
        {
            return Err(vec![ModuleValidationError::BibleNotFound(bible_name.clone())])
        };

        let mut all_refs = vec![];
        for l in &self.links
        {
            for w in &l.words
            {
                match w.range
                {
                    WordRange::Single(word) => {
                        all_refs.push(RefId::from_verse_id(l.verse_id, Some(word)));
                    },
                    WordRange::Range(start, end) => {
                        all_refs.push(RefId::from_verse_id(l.verse_id, Some(start)));
                        all_refs.push(RefId::from_verse_id(l.verse_id, Some(end)));
                    },
                }
            }
        }

        let mut errors = vec![];
        for r in all_refs
        {
            if !bible.source.id_exists(&r)
            {
                errors.push(ModuleValidationError::RefIdDoesNotExist(r, bible_name.clone()));
            }
        }

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(())    
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrongsLinkEntry 
{
    pub verse_id: VerseId,
    pub id: u32,
    pub words: Vec<StrongsWord>,
}

impl StrongsLinkEntry
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrongsWord
{
    pub strongs: StrongsNumber,
    pub range: WordRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordRange
{
    Single(NonZeroU32),
    Range(NonZeroU32, NonZeroU32),
}

impl Display for WordRange
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            Self::Single(i) => write!(f, "{}", i),
            Self::Range(s, e) => write!(f, "{}-{}", s, e)
        }
    }
}

impl FromStr for WordRange
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        if let Some(caps) = WORD_INDEX_REGEX.captures(s) 
        {
            let index = caps.name("index").unwrap().as_str().parse::<NonZeroU32>().unwrap();
            Ok(Self::Single(index))
        }
        else if let Some(caps) = WORD_RANGE_REGEX.captures(s)
        {
            let start = caps.name("start").unwrap().as_str().parse::<NonZeroU32>().unwrap();
            let end = caps.name("end").unwrap().as_str().parse::<NonZeroU32>().unwrap();
            Ok(Self::Range(start, end))
        }
        else 
        {
            Err(format!("`{}` is not a word range", s))
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
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}