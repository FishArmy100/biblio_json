pub mod highlight_color;

use serde::{Deserialize, Serialize};

use crate::{core::{RefId, lang::Language}, html_text::HtmlText, modules::{EntryId, ExternalModuleData, notebook::highlight_color::HighlightColor}};

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

    pub links_path: String,
    
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