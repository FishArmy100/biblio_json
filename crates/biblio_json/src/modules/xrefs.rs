use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::{RefId, lang::Language}, html_text::HtmlText, modules::{ModuleValidationContext, ModuleValidationError, bible::BibleModule}, utils};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct XRefsConfig
{
    pub name: String,
    pub authors: Option<Vec<String>>,
    pub language: Option<Language>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub license: Option<String>,
    pub bible: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct MutualRef
{
    pub id: RefId,
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum XRef 
{
    Directed 
    {
        source: RefId,
        targets: Vec<RefId>,
        note: Option<String>,
    },
    Mutual 
    {
        refs: Vec<MutualRef>,
        note: Option<String>,
    },
}

impl XRef
{
    pub fn from_file(path: &str) -> Result<Vec<Self>, String>
    {
        let ret = utils::load_json_lines(path)?
            .into_iter()
            .map(|(l, _)| l)
            .collect();

        Ok(ret)
    }

    pub fn has_source(&self, id: &RefId) -> bool
    {
        match self 
        {
            Self::Directed { source, targets: _, note: _ } => source == id,
            Self::Mutual { refs, note: _ } => refs.iter().find(|r| r.id == *id).is_some()
        }
    }

    pub fn collect_ref_ids(&self) -> Vec<RefId>
    {
        match self {
            XRef::Directed { source, targets, note: _ } => {
                let mut ids = targets.iter().map(|t| t.clone()).collect_vec();
                ids.push(source.clone());
                ids
            },
            XRef::Mutual { refs, note: _ } => refs.iter().map(|r| r.id.clone()).collect_vec(),
        }
    }
}

#[derive(Debug)]
pub struct XRefModule
{
    pub config: XRefsConfig,
    pub refs: Vec<XRef>,
}

impl XRefModule
{
    pub fn load(dir_path: &str, name: &str) -> Result<Self, String>
    {
        let config_path = format!("{}/{}.toml", dir_path, name);
        let config: XRefsConfig = utils::load_toml(config_path)?;

        let dictionary_path = format!("{}/{}.jsonl", dir_path, name);
        let refs = XRef::from_file(&dictionary_path)?;

        Ok(Self { 
            config,
            refs,
        })
    }

    pub fn validate(&self, context: &ModuleValidationContext) -> Result<(), Vec<ModuleValidationError>>
    {
        if let Some(bible_name) = &self.config.bible
        {
            let Some(bible) = context.bibles.get(bible_name) else
            {
                return Err(vec![ModuleValidationError::BibleNotFound(bible_name.clone())])
            };

            let mut errors = vec![];
            for r in self.refs.iter().map(|r| r.collect_ref_ids().into_iter()).flatten()
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
        else  
        {
            let mut errors = vec![];
            for r in self.refs.iter().map(|r| r.collect_ref_ids().into_iter()).flatten()
            {
                if r.is_word()
                {
                    errors.push(ModuleValidationError::WordRefIdInvalid(r));
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
}