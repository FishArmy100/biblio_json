pub mod bible;
pub mod dict;
pub mod xrefs;

use bible::BibleModule;

use crate::modules::dict::DictModule;



#[derive(Debug)]
pub enum Module
{
    Bible(BibleModule),
    Dictionary(DictModule)
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

    pub fn is_dict(&self) -> bool
    {
        match self 
        {
            Self::Dictionary(_) => true,
            _ => false,
        }
    }
}
