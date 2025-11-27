use itertools::{EitherOrBoth, Itertools};
use serde::{Deserialize, Serialize};

use crate::{core::lang::Language, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleId, ModuleValidationError}, utils, validation::ValidationContextBuilder};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictConfig
{
    pub name: String,
    pub id: ModuleId,
    pub short_name: Option<String>,
    pub authors: Option<Vec<String>>,
    pub language: Option<Language>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub license: Option<String>,
    #[serde(default)]
    pub external: ExternalModuleData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictEntry
{
    pub term: String,
    pub aliases: Option<Vec<String>>,
    pub definition: HtmlText,
    pub id: EntryId,
}

impl DictEntry
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
pub struct DictModule
{
    pub config: DictConfig,
    pub entries: Vec<DictEntry>,
}

impl DictModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: DictConfig = utils::load_toml(config_path)?;

        let dictionary_path = format!("{}/{}.jsonl", dir_path, name);
        let entries = DictEntry::from_file(&dictionary_path)?;

        Ok(Self { 
            config,
            entries,
        })
    }

    pub fn find(&self, term: &str) -> Option<&DictEntry>
    {
        self.entries.iter().find(|entry| {
            let contains_alias = entry.aliases.as_ref().is_some_and(|a| a.iter().find(|t| eq_ignore_punc_and_case(t, term)).is_some());
            eq_ignore_punc_and_case(&entry.term, term) || contains_alias
        })
    }

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        let context = builder.build(Some(&self.config.id), &self.config.external);
        
        let mut errors = self.config.description.as_ref()
            .map(|d| d.validate(&context).err())
            .flatten()
            .map(|errors| errors.into_iter().map(|error| ModuleValidationError::HtmlError { error }).collect_vec())
            .unwrap_or_default();

        let duplicates = utils::find_duplicates(self.entries.iter().map(|e| e.id))
            .map(|d| ModuleValidationError::EntryIdDuplicate { id: d })
            .collect_vec();

        errors.extend(duplicates);

        for entry in &self.entries
        {
            let entry_errors = entry.definition.validate(&context).err()
                .unwrap_or_default()
                .into_iter()
                .map(|error| ModuleValidationError::HtmlError { error })
                .collect_vec();

            errors.extend(entry_errors);
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

fn eq_ignore_punc_and_case(a: &str, b: &str) -> bool
{
    let a_chars = get_normalized_str_chars(a);
    let b_chars = get_normalized_str_chars(b);

    for pair in a_chars.zip_longest(b_chars)
    {
        let EitherOrBoth::Both(a, b) = pair else {
            return false;
        };

        if a != b 
        {
            return false
        }
    }

    true
}

fn get_normalized_str_chars(s: &str) -> impl Iterator<Item = char>
{
    s.chars().into_iter()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .map(|c| c.to_ascii_lowercase())
}