pub mod bible;
pub mod dict;
pub mod xrefs;
pub mod link;

use bible::BibleModule;

use crate::modules::{dict::DictModule, link::LinkerModule, xrefs::XRefModule};



#[derive(Debug)]
pub enum Module
{
    Bible(BibleModule),
    Dictionary(DictModule),
    XRef(XRefModule),
    Linker(LinkerModule)
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

    pub fn as_bible(&self) -> Option<&BibleModule>
    {
        match self 
        {
            Self::Bible(b) => Some(b),
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

    pub fn as_dict(&self) -> Option<&DictModule>
    {
        match self
        {
            Self::Dictionary(d) => Some(d),
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

    pub fn as_xrefs(&self) -> Option<&XRefModule>
    {
        match self
        {
            Self::XRef(x) => Some(x),
            _ => None,
        }
    }

    pub fn is_linker(&self) -> bool 
    {
        match self 
        {
            Self::Linker(_) => true,
            _ => false,
        }
    }
    
    pub fn as_linker(&self) -> Option<&LinkerModule>
    {
        match self
        {
            Self::Linker(l) => Some(l),
            _ => None,
        }
    }

    pub fn get_name(&self) -> &str 
    {
        match self 
        {
            Module::Bible(bible_module) => &bible_module.name,
            Module::Dictionary(dict_module) => &dict_module.name,
            Module::XRef(xref_module) => &xref_module.name,
            Module::Linker(linker_module) => &linker_module.name,
        }
    }
}
