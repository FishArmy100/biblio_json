use std::{collections::HashMap, sync::Arc};

use crate::{core::RefId, modules::bible::BibleModule};





#[derive(Debug, Clone)]
pub struct ValidationContext<'a>
{
    pub bibles: &'a HashMap<String, Arc<BibleModule>>,
}

impl<'a> ValidationContext<'a>
{
    pub fn ref_id_exists(&self, id: RefId, default_bible: Option<&str>) -> bool
    {
        if let Some(default_bible) = default_bible.map(|b| self.bibles.get(b)).flatten()
        {

        }
        else if let Some(bible) = id.bible.as_ref().map(|b| self.bibles.get(b)).flatten()
        {

        }
        else 
        {
                
        }
    }
}