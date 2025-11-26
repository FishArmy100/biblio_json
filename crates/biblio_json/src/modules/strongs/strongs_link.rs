use std::{collections::HashMap};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::{RefId, StrongsNumber, VerseId, WordRange, lang::Language}, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleValidationError}, utils, validation::ValidationContextBuilder};

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

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        let context = builder.build(Some(&self.config.bible), &self.config.external);

        let bible_name = &self.config.bible;
        let Some(_) = context.bibles.get(bible_name) else
        {
            return Err(vec![ModuleValidationError::BibleNotFound {
                name: bible_name.clone(),
            }])
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

        let duplicate_errors = utils::find_duplicates(self.entries.iter().map(|e| e.id))
            .map(|d| ModuleValidationError::EntryIdDuplicate { id: d })
            .collect_vec();

        let config_errors = self.config.description.as_ref().iter()
            .flat_map(|d| d.validate(&context).err())
            .flatten()
            .map(|error| ModuleValidationError::HtmlError { error })
            .collect_vec();

        let ref_id_errors = all_refs.iter()
            .filter_map(|id| match context.validate_ref_id(id) {
                Ok(()) => None,
                Err(e) => Some((e, id))
            })
            .map(|(e, id)| ModuleValidationError::RefIdError { id: id.clone(), error: e })
            .collect_vec();

        let mut errors = vec![];
        errors.extend(config_errors);
        errors.extend(ref_id_errors);
        errors.extend(duplicate_errors);

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
        let ret = utils::load_json_lines(path).stringify_error()?
            .into_iter()
            .map(|l| l.value)
            .collect();

        Ok(ret)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrongsWord
{
    pub strongs: Vec<StrongsNumber>,
    pub primary: Option<StrongsNumber>,
    pub range: WordRange,
}