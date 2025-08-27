use serde::{Deserialize, Serialize};

use crate::{core::{RefId, lang::Language}, html_text::HtmlText, utils};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentaryConfig
{
    pub name: String,
    pub bible: Option<String>,
    pub authors: Option<Vec<String>>,
    pub language: Option<Language>,
    pub description: Option<HtmlText>,
    pub data_source: Option<String>,
    pub pub_year: Option<u32>,
    pub license: Option<String>,
}

#[derive(Debug)]
pub struct CommentaryModule
{
    pub config: CommentaryConfig,
    pub entries: Vec<CommentaryEntry>,
}

impl CommentaryModule
{
    pub fn load(dir_path: &str, name: &str) -> Result<Self, String>
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentaryEntry
{
    pub id: u32,
    pub references: Vec<RefId>,
    pub comment: HtmlText,
}

impl CommentaryEntry
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