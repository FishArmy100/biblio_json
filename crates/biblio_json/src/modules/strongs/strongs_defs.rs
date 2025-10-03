use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::{lang::Language, strongs_number::StrongsNumber}, html_text::HtmlText, modules::{EntryId, ExternalModuleData, ModuleValidationError}, utils, validation::ValidationContextBuilder};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StrongsDefEntry
{
    pub strongs_ref: StrongsNumber,
    pub word: String,
    pub definitions: Vec<HtmlText>,
    pub derivation: Option<HtmlText>,
    pub id: EntryId,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrongsDefsConfig
{
    pub name: String,
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
pub struct StrongsDefsModule
{
    pub config: StrongsDefsConfig,
    pub index_map: HashMap<StrongsNumber, u32>,
    pub entries: Vec<StrongsDefEntry>,
}

impl StrongsDefsModule
{
    pub fn load_json(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: StrongsDefsConfig = utils::load_toml(config_path)?;

        let bible_path = format!("{}/{}.jsonl", dir_path, name);
        let defs = StrongsDefEntry::from_file(&bible_path)?;

        let index_map: HashMap<_, _> = defs.iter().enumerate().map(|(i, d)| {
            (d.strongs_ref.clone(), i as u32)
        }).collect();

        Ok(Self { 
            config,
            index_map,
            entries: defs,
        })
    }

    pub fn get_def(&self, num: &StrongsNumber) -> Option<&StrongsDefEntry>
    {
        match self.index_map.get(num)
        {
            Some(idx) => Some(&self.entries[*idx as usize]),
            None => None
        }
    }

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        let context = builder.build(None, &self.config.external);

        let config_errors = self.config.description.as_ref().iter()
            .flat_map(|d| d.validate(&context).err())
            .flatten()
            .map(|error| ModuleValidationError::HtmlError { error })
            .collect_vec();

        let entry_errors = self.entries.iter().map(|e| {
            let mut errors = e.definitions.iter().filter_map(|d| d.validate(&context).err()).flatten().collect_vec();
            if let Some(derivation) = &e.derivation
            {
                if let Some(e) = derivation.validate(&context).err()
                {
                    errors.extend(e);
                }
            }
            
            errors
        })
        .flatten()
        .map(|e| ModuleValidationError::HtmlError { error: e })
        .collect_vec();

        let mut errors = vec![];
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

impl StrongsDefEntry
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