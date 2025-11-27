use std::num::NonZeroU32;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::{RefId, VerseId, lang::Language}, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleId, ModuleValidationError}, utils, validation::ValidationContextBuilder};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommentaryConfig
{
    pub name: String,
    pub id: ModuleId,
    pub short_name: Option<String>,
    pub bible: Option<ModuleId>,
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
pub struct CommentaryModule
{
    pub config: CommentaryConfig,
    pub entries: Vec<CommentaryEntry>,
}

impl CommentaryModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: CommentaryConfig = utils::load_toml(config_path)?;

        let dictionary_path = format!("{}/{}.jsonl", dir_path, name);
        let entries = CommentaryEntry::from_file(&dictionary_path)?;

        Ok(Self { 
            config,
            entries,
        })
    }

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        let context = builder.build(self.config.bible.as_ref(), &self.config.external);

        if let Some(bible) = self.config.bible.as_ref()
        {
            if let None = context.bibles.get(bible)
            {
                return Err(vec![ModuleValidationError::BibleNotFound {
                    name: bible.clone()
                }]);
            }
        }

        let duplicates = utils::find_duplicates(self.entries.iter().map(|e| e.id))
            .map(|d| ModuleValidationError::EntryIdDuplicate { id: d })
            .collect_vec();

        let config_errors = self.config.description.as_ref().iter()
            .flat_map(|d| d.validate(&context).err())
            .flatten()
            .map(|error| ModuleValidationError::HtmlError { error })
            .collect_vec();

        let entry_errors = self.entries.iter().map(|e| {
            let mut errors = vec![];
            if let Err(e) = e.comment.validate(&context)
            {
                errors.extend(e.into_iter().map(|e| ModuleValidationError::HtmlError { error: e }));
            }

            let ref_id_errors = e.references.iter()
                .filter_map(|id| match context.validate_ref_id(id) {
                    Ok(()) => None,
                    Err(e) => Some((e, id))
                })
                .map(|(e, id)| ModuleValidationError::RefIdError { id: id.clone(), error: e })
                .collect_vec();

            errors.extend(ref_id_errors);
            errors
        }).flatten().collect_vec();

        let mut errors = vec![];
        errors.extend(config_errors);
        errors.extend(entry_errors);
        errors.extend(duplicates);

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommentaryEntry
{
    pub id: EntryId,
    pub references: Vec<RefId>,
    pub comment: HtmlText,
}

impl CommentaryEntry
{
    pub fn from_file(path: &str) -> Result<Vec<Self>, String>
    {
        let ret = utils::load_json_lines(path).stringify_error()?
            .into_iter()
            .map(|l| l.value)
            .collect();

        Ok(ret)
    }

    pub fn has_verse(&self, verse: &VerseId) -> bool
    {
        self.references.iter().any(|r| r.has_verse(verse))
    }

    pub fn has_verse_word(&self, verse: &VerseId, word: NonZeroU32) -> bool
    {
        self.references.iter().any(|r| r.has_verse_word(verse, word))
    }
}