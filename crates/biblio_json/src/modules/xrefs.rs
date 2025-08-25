use serde::{Deserialize, Serialize};

use crate::{core::RefId, utils};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct XRefsConfig
{
    pub name: String,
    pub description: Option<String>,
    pub data_source: Option<String>,
    pub license: Option<String>,
    pub language: Option<String>,
    pub pub_year: Option<u32>,
    pub bible_dep: Option<String>,
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
        source_text: Option<String>,
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
            Self::Directed { source, source_text: _, targets: _, note: _ } => source == id,
            Self::Mutual { refs, note: _ } => refs.iter().find(|r| r.id == *id).is_some()
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
}