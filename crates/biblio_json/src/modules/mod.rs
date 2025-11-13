pub mod bible;
pub mod dict;
pub mod xrefs;
pub mod strongs;
pub mod commentary;
pub mod notebook;
pub mod readings;

use std::{collections::HashMap, fmt::Display, sync::Arc};

use bible::BibleModule;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{core::{RefId, lang::Language}, html_text::{HtmlText, HtmlValidationError, ast::AssetIdName}, modules::{bible::Verse, commentary::{CommentaryEntry, CommentaryModule}, dict::{DictEntry, DictModule}, notebook::{NotebookEntry, NotebookModule}, readings::{ReadingsEntry, ReadingsModule, ReadingsModuleValidationError}, strongs::{StrongsDefEntry, StrongsDefsModule, StrongsLinkEntry, StrongsLinksModule}, xrefs::{XRefEntry, XRefModule}}, validation::{RefIdValidationError, ValidationContextBuilder}};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ExternalModuleData
{
    #[serde(default)]
    pub aliases: HashMap<AssetIdName, String>,
    #[serde(default)]
    pub assets: HashMap<AssetIdName, String>,
}

#[derive(Debug)]
pub enum ModuleValidationError
{
    BibleNotFound
    {
        name: String,
    },
    RefIdError
    {
        id: RefId,
        error: RefIdValidationError,
    },
    HtmlError 
    {
        error: HtmlValidationError,
    },
    EntryIdDuplicate
    {
        id: EntryId,
    },
    ReadingsModuleError(ReadingsModuleValidationError)
}

impl From<ReadingsModuleValidationError> for ModuleValidationError
{
    fn from(error: ReadingsModuleValidationError) -> Self 
    {
        Self::ReadingsModuleError(error)
    }
}

impl Display for ModuleValidationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            Self::BibleNotFound { name } => write!(f, "Bible '{}' does not exist.", name),
            Self::RefIdError { id, error } => match error {
                RefIdValidationError::DoesNotExist => write!(f, "RefId {} does not exist in bible in the current context", id),
                RefIdValidationError::NeedsBible => write!(f, "RefId {} needs a bible reference", id),
            },
            Self::HtmlError { error } => write!(f, "{}", error),
            Self::EntryIdDuplicate { id } => write!(f, "Duplicate entries for id '{}'", id),
            Self::ReadingsModuleError(e) => write!(f, "{}", e),
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Module
{
    Bible(#[serde_as(as = "Arc<_>")] Arc<BibleModule>),
    Dictionary(#[serde_as(as = "Arc<_>")] Arc<DictModule>),
    XRef(#[serde_as(as = "Arc<_>")] Arc<XRefModule>),
    StrongsDefs(#[serde_as(as = "Arc<_>")] Arc<StrongsDefsModule>),
    StrongsLinks(#[serde_as(as = "Arc<_>")] Arc<StrongsLinksModule>),
    Commentary(#[serde_as(as = "Arc<_>")] Arc<CommentaryModule>),
    Notebook(#[serde_as(as = "Arc<_>")] Arc<NotebookModule>),
    Readings(#[serde_as(as = "Arc<_>")] Arc<ReadingsModule>),
}

impl Module
{
    pub fn is_bible(&self) -> bool
    {
        match self 
        {
            Self::Bible(_) => true,
            _ => false,
        }
    }

    pub fn as_bible(&self) -> Option<Arc<BibleModule>>
    {
        match self 
        {
            Self::Bible(b) => Some(b.clone()),
            _ => None,
        }
    }

    pub fn is_dict(&self) -> bool
    {
        match self 
        {
            Self::Dictionary(_) => true,
            _ => false,
        }
    }

    pub fn as_dict(&self) -> Option<Arc<DictModule>>
    {
        match self
        {
            Self::Dictionary(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn is_xrefs(&self) -> bool
    {
        match self 
        {
            Self::XRef(_) => true,
            _ => false,
        }
    }

    pub fn as_xrefs(&self) -> Option<Arc<XRefModule>>
    {
        match self
        {
            Self::XRef(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn is_strongs_defs(&self) -> bool
    {
        match self 
        {
            Self::StrongsDefs(_) => true,
            _ => false,
        }
    }

    pub fn as_strongs_defs(&self) -> Option<Arc<StrongsDefsModule>>
    {
        match self 
        {
            Self::StrongsDefs(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn is_strongs_links(&self) -> bool
    {
        match self 
        {
            Self::StrongsLinks(_) => true,
            _ => false,
        }
    }
    
    pub fn as_strongs_links(&self) -> Option<Arc<StrongsLinksModule>>
    {
        match self 
        {
            Self::StrongsLinks(m) => Some(m.clone()),
            _ => None,
        }
    }

    pub fn is_commentary(&self) -> bool
    {
        match self 
        {
            Self::Commentary(_) => true,
            _ => false,
        }
    }

    pub fn as_commentary(&self) -> Option<Arc<CommentaryModule>>
    {
        match self 
        {
            Self::Commentary(c) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn is_notebook(&self) -> bool
    {
        match self 
        {
            Self::Notebook(_) => true,
            _ => false,
        }
    }

    pub fn as_notebook(&self) -> Option<Arc<NotebookModule>>
    {
        match self 
        {
            Self::Notebook(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub fn is_readings(&self) -> bool
    {
        match self 
        {
            Self::Readings(_) => true,
            _ => false
        }
    }

    pub fn as_readings(&self) -> Option<Arc<ReadingsModule>>
    {
        match self 
        {
            Self::Readings(r) => Some(r.clone()),
            _ => None,
        }
    }



    pub fn name(&self) -> &str 
    {
        match self 
        {
            Module::Bible(bible_module) => &bible_module.config.name,
            Module::Dictionary(dict_module) => &dict_module.config.name,
            Module::XRef(xref_module) => &xref_module.config.name,
            Module::StrongsDefs(strongs) => &strongs.config.name,
            Module::StrongsLinks(strongs_links_module) => &strongs_links_module.config.name,
            Module::Commentary(commentary_module) => &commentary_module.config.name,
            Module::Notebook(notebook_module) => &notebook_module.config.name,
            Module::Readings(reading) => &reading.config.name
        }
    }

    pub fn description(&self) -> Option<&HtmlText>
    {
        match self 
        {
            Module::Bible(bible_module) => bible_module.config.description.as_ref(),
            Module::Dictionary(dict_module) => dict_module.config.description.as_ref(),
            Module::XRef(xref_module) => xref_module.config.description.as_ref(),
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.description.as_ref(),
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.description.as_ref(),
            Module::Commentary(commentary_module) => commentary_module.config.description.as_ref(),
            Module::Notebook(notebook_module) => notebook_module.config.description.as_ref(),
            Module::Readings(readings_module) => readings_module.config.description.as_ref(),
        }
    }

    pub fn language(&self) -> Option<Language>
    {
        match self
        {
            Module::Bible(bible_module) => bible_module.config.language,
            Module::Dictionary(dict_module) => dict_module.config.language,
            Module::XRef(xref_module) => xref_module.config.language,
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.language,
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.language,
            Module::Commentary(commentary_module) => commentary_module.config.language,
            Module::Notebook(notebook_module) => notebook_module.config.language,
            Module::Readings(readings_module) => readings_module.config.language,
        }
    }

    pub fn authors(&self) -> Option<&Vec<String>>
    {
        match self
        {
            Module::Bible(bible_module) => bible_module.config.authors.as_ref(),
            Module::Dictionary(dict_module) => dict_module.config.authors.as_ref(),
            Module::XRef(xref_module) => xref_module.config.authors.as_ref(),
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.authors.as_ref(),
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.authors.as_ref(),
            Module::Commentary(commentary_module) => commentary_module.config.authors.as_ref(),
            Module::Notebook(notebook_module) => notebook_module.config.authors.as_ref(),
            Module::Readings(readings_module) => readings_module.config.authors.as_ref(),
        }
    }

    pub fn pub_year(&self) -> Option<u32>
    {
        match self
        {
            Module::Bible(bible_module) => bible_module.config.pub_year,
            Module::Dictionary(dict_module) => dict_module.config.pub_year,
            Module::XRef(xref_module) => xref_module.config.pub_year,
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.config.pub_year,
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.config.pub_year,
            Module::Commentary(commentary_module) => commentary_module.config.pub_year,
            Module::Notebook(notebook_module) => notebook_module.config.pub_year,
            Module::Readings(readings_module) => readings_module.config.pub_year,
        }
    }

    pub fn external(&self) -> &ExternalModuleData
    {
        match self
        {
            Module::Bible(bible_module) => &bible_module.config.external,
            Module::Dictionary(dict_module) => &dict_module.config.external,
            Module::XRef(xref_module) => &xref_module.config.external,
            Module::StrongsDefs(strongs_defs_module) => &strongs_defs_module.config.external,
            Module::StrongsLinks(strongs_links_module) => &strongs_links_module.config.external,
            Module::Commentary(commentary_module) => &commentary_module.config.external,
            Module::Notebook(notebook_module) => &notebook_module.config.external,
            Module::Readings(readings_module) => &readings_module.config.external,
        }
    }

    pub fn validate(&self, builder: &ValidationContextBuilder) -> Result<(), Vec<ModuleValidationError>>
    {
        match self 
        {
            Module::Bible(bible) => bible.validate(builder),
            Module::Dictionary(dict) => dict.validate(builder),
            Module::XRef(xref_module) => xref_module.validate(&builder),
            Module::StrongsDefs(defs) => defs.validate(builder),
            Module::StrongsLinks(links) => links.validate(builder),
            Module::Commentary(commentary) => commentary.validate(builder),
            Module::Notebook(notebook_module) => notebook_module.validate(builder),
            Module::Readings(readings_module) => readings_module.validate(builder),
        }
    }

    pub fn has_entry(&self, entry: EntryId) -> bool
    {
        match self 
        {
            Module::Bible(bible_module) => bible_module.source.verses.values().find(|e| e.id == entry).is_some(),
            Module::Dictionary(dict_module) => dict_module.entries.iter().find(|e| e.id == entry).is_some(),
            Module::XRef(xref_module) => xref_module.entries.iter().find(|e| e.id() == entry).is_some(),
            Module::StrongsDefs(strongs_defs_module) => strongs_defs_module.entries.iter().find(|e| e.id == entry).is_some(),
            Module::StrongsLinks(strongs_links_module) => strongs_links_module.entries.iter().find(|e| e.id == entry).is_some(),
            Module::Commentary(commentary_module) => commentary_module.entries.iter().find(|e| e.id == entry).is_some(),
            Module::Notebook(notebook_module) => notebook_module.entries.iter().find(|e| e.id() == entry).is_some(),
            Module::Readings(readings_module) => readings_module.entries.iter().find(|e| e.id == entry).is_some()
        }
    }

    pub fn get_entry(&'_ self, entry_id: EntryId) -> Option<ModuleEntry<'_>>
    {
        let entry = match self 
        {
            Module::Bible(bible) => ModuleEntry::Verse(bible.source.verses.values().find(|v| v.id == entry_id)?),
            Module::Dictionary(dict) => ModuleEntry::Dictionary(dict.entries.iter().find(|e| e.id == entry_id)?),
            Module::XRef(xref) => ModuleEntry::XRef(xref.entries.iter().find(|e| e.id() == entry_id)?),
            Module::StrongsDefs(strongs_defs) => ModuleEntry::StrongsDef(strongs_defs.entries.iter().find(|e| e.id == entry_id)?),
            Module::StrongsLinks(strongs_links) => ModuleEntry::StrongsLink(strongs_links.entries.iter().find(|e| e.id == entry_id)?),
            Module::Commentary(commentary) => ModuleEntry::Commentary(commentary.entries.iter().find(|e| e.id == entry_id)?),
            Module::Notebook(notebook) => ModuleEntry::Notebook(notebook.entries.iter().find(|e| e.id() == entry_id)?),
            Module::Readings(readings_module) => ModuleEntry::Readings(readings_module.entries.iter().find(|e| e.id == entry_id)?),
        };

        Some(entry)
    }
}

pub type EntryId = u32;

#[derive(Debug, Clone, Copy)]
pub enum ModuleEntry<'a>
{
    Dictionary(&'a DictEntry),
    StrongsDef(&'a StrongsDefEntry),
    StrongsLink(&'a StrongsLinkEntry),
    XRef(&'a XRefEntry),
    Commentary(&'a CommentaryEntry),
    Verse(&'a Verse),
    Notebook(&'a NotebookEntry),
    Readings(&'a ReadingsEntry),
}

impl<'a> ModuleEntry<'a>
{
    pub fn is_dictionary(&self) -> bool 
    {
        matches!(self, Self::Dictionary(_))
    }

    pub fn as_dictionary(&self) -> Option<&'a DictEntry> 
    {
        match self 
        {
            Self::Dictionary(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_strongs_def(&self) -> bool 
    {
        matches!(self, Self::StrongsDef(_))
    }

    pub fn as_strongs_def(&self) -> Option<&'a StrongsDefEntry> 
    {
        match self 
        {
            Self::StrongsDef(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_strongs_link(&self) -> bool 
    {
        matches!(self, Self::StrongsLink(_))
    }

    pub fn as_strongs_link(&self) -> Option<&'a StrongsLinkEntry> 
    {
        match self 
        {
            Self::StrongsLink(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_xref(&self) -> bool 
    {
        matches!(self, Self::XRef(_))
    }

    pub fn as_xref(&self) -> Option<&'a XRefEntry> 
    {
        match self 
        {
            Self::XRef(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_commentary(&self) -> bool 
    {
        matches!(self, Self::Commentary(_))
    }

    pub fn as_commentary(&self) -> Option<&'a CommentaryEntry> 
    {
        match self 
        {
            Self::Commentary(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_verse(&self) -> bool 
    {
        matches!(self, ModuleEntry::Verse(_))
    }

    pub fn as_verse(&self) -> Option<&'a Verse> 
    {
        match self 
        {
            ModuleEntry::Verse(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_notebook(&self) -> bool 
    {
        matches!(self, ModuleEntry::Notebook(_))
    }

    pub fn as_notebook(&self) -> Option<&'a NotebookEntry> 
    {
        match self 
        {
            ModuleEntry::Notebook(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_readings(&self) -> bool
    {
        matches!(self, ModuleEntry::Readings(_))
    }

    pub fn as_readings(&self) -> Option<&'a ReadingsEntry>
    {
        match self 
        {
            ModuleEntry::Readings(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleEntryRef
{
    pub module: String,
    pub entry_id: EntryId,
}
