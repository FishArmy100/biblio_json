pub mod highlight_color;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::{RefId, lang::Language}, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleValidationError, notebook::highlight_color::HighlightColor}, utils, validation::ValidationContextBuilder};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct NotebookConfig
{
    pub name: String,
    pub authors: Option<Vec<String>>,
    pub language: Option<Language>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub bible: Option<String>,
    
    #[serde(default)]
    pub external: ExternalModuleData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
#[serde(deny_unknown_fields)]
pub enum NotebookEntry
{
    Highlight
    {
        id: EntryId,
        name: String,
        description: Option<HtmlText>,
        priority: u32,
        color: HighlightColor,
        references: Vec<RefId>,
    },
    Note 
    {
        id: EntryId,
        name: Option<String>,
        content: HtmlText,
        references: Vec<RefId>,
    }
}

impl NotebookEntry
{
    pub fn id(&self) -> EntryId
    {
        match self 
        {
            NotebookEntry::Highlight { id, .. } => *id,
            NotebookEntry::Note { id, .. } => *id,
        }
    }
}

impl NotebookEntry
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

#[derive(Debug, Serialize, Deserialize)]
pub struct NotebookModule
{
    pub config: NotebookConfig,
    pub entries: Vec<NotebookEntry>,
}

impl NotebookModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: NotebookConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let entries = NotebookEntry::from_file(&bible_path)?;

        Ok(Self { 
            config,
            entries,
        })
    }

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        let context = builder.build(self.config.bible.as_ref().map(|e| e.as_str()), &self.config.external);

        if let Some(bible) = self.config.bible.as_ref()
        {
            if let None = context.bibles.get(bible)
            {
                return Err(vec![ModuleValidationError::BibleNotFound {
                    name: bible.clone()
                }]);
            }
        }

        let duplicate_errors = utils::find_duplicates(self.entries.iter().map(|e| e.id()))
            .map(|d| ModuleValidationError::EntryIdDuplicate { id: d })
            .collect_vec();

        let config_errors = self.config.description.as_ref().iter()
            .flat_map(|d| d.validate(&context).err())
            .flatten()
            .map(|error| ModuleValidationError::HtmlError { error })
            .collect_vec();

        let entry_errors = self.entries.iter().map(|e| {
            match e 
            {
                NotebookEntry::Highlight { description, references, .. } => {
                    let mut errors = vec![];

                    let desc_errors = description.as_ref().iter()
                        .flat_map(|d| d.validate(&context).err())
                        .flatten()
                        .map(|error| ModuleValidationError::HtmlError { error })
                        .collect_vec();

                    let ref_errors = references.iter()
                        .filter_map(|id| match context.validate_ref_id(id) {
                            Ok(()) => None,
                            Err(e) => Some((e, id))
                        })
                        .map(|(e, id)| ModuleValidationError::RefIdError { id: id.clone(), error: e })
                        .collect_vec();
                    
                    errors.extend(desc_errors);
                    errors.extend(ref_errors);
                    errors
                },
                NotebookEntry::Note { references, .. } => {
                    references.iter()
                        .filter_map(|id| match context.validate_ref_id(id) {
                            Ok(()) => None,
                            Err(e) => Some((e, id))
                        })
                        .map(|(e, id)| ModuleValidationError::RefIdError { id: id.clone(), error: e })
                        .collect_vec()
                },
            }
        }).flatten().collect_vec();

        let mut errors = vec![];
        errors.extend(duplicate_errors);
        errors.extend(config_errors);
        errors.extend(entry_errors);

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