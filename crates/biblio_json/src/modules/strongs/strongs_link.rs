use std::{collections::HashMap, fmt::Display, num::NonZeroU32, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{core::{RefId, StrongsNumber, VerseId, WordRange, lang::Language}, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleValidationContext, ModuleValidationError}, utils};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct StrongsLinksModule
{
    pub config: StrongsLinkConfig,
    pub index_map: HashMap<VerseId, u32>,
    pub entries: Vec<StrongsLinkEntry>,
}

impl StrongsLinksModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
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
            entries: links,
        })
    }

    pub fn get_links(&self, verse: &VerseId) -> Option<&StrongsLinkEntry>
    {
        match self.index_map.get(verse)
        {
            Some(idx) => Some(&self.entries[*idx as usize]),
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
        for l in &self.entries
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
    pub id: EntryId,
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