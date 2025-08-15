pub(crate) mod utils;
pub mod modules;
pub mod ref_id;
use std::{collections::HashMap, fmt::Display, path::Path};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{modules::{bible::{BibleModule, Verse}, dict::{DictEntry, DictModule}, link::{LinkerModule, WordRange}, strongs::StrongsDefsModule, xrefs::{XRef, XRefModule}, Module}, ref_id::{Atom, RefId}};

pub const PACKAGE_FILE_NAME: &str = "biblio-json.toml";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PackageConfig
{
    pub name: String,
    pub authors: Vec<String>,
    pub license: String,
    pub module_paths: Option<ModulePaths>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ModulePaths
{
    pub bibles: Option<String>,
    pub dictionaries: Option<String>,
    pub xrefs: Option<String>,
    pub linkers: Option<String>,
    pub strongs_defs: Option<String>,
}

pub enum PackageValidationError
{
    InvalidRefId
    {
        id: RefId,
        bible_name: String,
        xref_name: String,
        line: usize,
    }
}

impl Display for PackageValidationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            Self::InvalidRefId { id, bible_name, xref_name, line } => {
                write!(f, "RefId {} in xref module {} on line {} does not exist in Bible {}", id, xref_name, line, bible_name)
            }
        }
    }
}

#[derive(Debug)]
pub struct FetchEntry<T>
{
    pub loc: RefId,
    pub module_name: String,
    pub entry: T,
}

#[derive(Debug)]
pub struct FetchData 
{
    pub verse: Verse,
    pub xrefs: Vec<FetchEntry<XRef>>,
    pub defs: Vec<FetchEntry<DictEntry>>,
}

#[derive(Debug)]
pub struct Package 
{
    pub name: String,
    pub authors: Vec<String>,
    pub license: String,
    pub modules: HashMap<String, Module>
}

impl Package
{
    pub fn load(dir_path: &str) -> Result<Self, Vec<String>>
    {
        let path = Path::new(dir_path);

        if !path.is_dir()
        {
            return Err(vec![format!("Provided path: {dir_path}, must be a directory")]);
        }

        let config_path = path.join(Path::new(PACKAGE_FILE_NAME));
        let file = utils::load_file(config_path).map_err(|e| vec![e])?;
        let config = match toml::from_str::<PackageConfig>(&file) {
            Ok(ok) => ok,
            Err(e) => return Err(vec![e.to_string()])
        };

        let modules = match &config.module_paths {
            Some(paths) => Self::load_modules(dir_path, paths)?,
            None => vec![]
        };

        Ok(Self {
            name: config.name,
            authors: config.authors,
            license: config.license,
            modules: modules.into_iter().map(|m| (m.get_name().to_owned(), m)).collect()
        })
    }

    pub fn get_mod(&self, name: &str) -> Option<&Module>
    {
        self.modules.get(name)
    }

    pub fn fetch(&self, bible_name: &str, ref_id: &RefId) -> Result<FetchData, String>
    {
        if !ref_id.is_verse() || ref_id.is_range() {
            return Err(format!("RefId {} is not in the format Gen.1.1", ref_id))
        }

        let Some(bible) = self.get_mod(bible_name).map(|b| b.as_bible()).flatten() else {
            return Err(format!("Bible {} does not exist in this package", bible_name));
        };

        let Some(verse) = bible.source.verses.get(ref_id).cloned() else {
            return Err(format!("Verse {} does not exist in bible {}", ref_id, bible_name));
        };

        let Some(linker) = self.modules.values().find_map(|m| m.as_linker().filter(|linker| linker.config.bible == bible_name)) else {
            return Err(format!("Bible {} does not have a linker in this package", bible_name));
        };

        let referenced_modules = linker.config.ref_works.iter().map(|(name, id)| {
            let Some(module) = self.get_mod(name) else {
                return Err(format!("Module {} does not exist in package {}", name, self.name));
            };

            if !module.is_dict() && !module.is_xrefs()
            {
                return Err(format!("Module `{}` in package `{}` is not a Dictionary or XRef module", module.get_name(), self.name))
            }

            Ok((*id, module))
        }).collect::<Result<HashMap<_, _>, _>>()?;

        let mut xrefs = vec![];
        let mut defs = vec![];

        if let Some(verse_link) = linker.links.get(ref_id)
        {
            for (r, link) in &verse_link.links
            {
                let loc = extract_ref_loc(ref_id, &verse, r);

                for link_ref in link
                {
                    let Some(module) = referenced_modules.get(&link_ref.ref_work_id) else {
                        return Err(format!("ref_work_id `{}` does not exist in package `{}`", link_ref.ref_work_id, self.name))
                    };

                    match module
                    {
                        Module::Dictionary(dict_module) => {
                            let Some(entry) = dict_module.entries.get(link_ref.work_index as usize) else {
                                return Err(format!("work_index `{}` is out of range for module `{}` in package `{}`", link_ref.work_index, dict_module.config.name, self.name));
                            };

                            defs.push(FetchEntry { 
                                loc: loc.clone(), 
                                module_name: dict_module.config.name.clone(), 
                                entry: entry.clone() 
                            });
                        },
                        Module::XRef(xref_module) => {
                            let Some(xref) = xref_module.refs.get(link_ref.work_index as usize) else {
                                return Err(format!("work_index {} is out of range for module {} in package {}", link_ref.work_index, xref_module.config.name, self.name));
                            };

                            xrefs.push(FetchEntry { 
                                loc: loc.clone(), 
                                module_name: xref_module.config.name.clone(), 
                                entry: xref.clone() 
                            });
                        },
                        Module::Bible(_) | Module::Linker(_) | Module::Strongs(_) => panic!("This should not be reachable"),
                    }
                }
            }
        }

        
        Ok(FetchData { 
            verse,
            xrefs,
            defs
        })
    }

    fn load_modules(root: &str, paths: &ModulePaths) -> Result<Vec<Module>, Vec<String>>
    {
        let mut modules = vec![];
        let mut errors = vec![];
        
        if let Some(bibles_path) = &paths.bibles
        {
            let result = Self::load_module(root, &bibles_path, |dir, name| 
            {
                Ok(Module::Bible(BibleModule::load(dir, name)?))
            });

            match result
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(dictionary_paths) = &paths.dictionaries
        {
            let result =  Self::load_module(root, &dictionary_paths, |dir, name| 
            {
                Ok(Module::Dictionary(DictModule::load(dir, name)?))
            });

            match result
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(xref_paths) = &paths.xrefs
        {
            let result =  Self::load_module(root, &xref_paths, |dir, name| 
            {
                Ok(Module::XRef(XRefModule::load(dir, name)?))
            });

            match result
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(linker_paths) = &paths.linkers
        {
            let result = Self::load_module(root, &linker_paths, |dir, name| {
                Ok(Module::Linker(LinkerModule::load(dir, name)?))
            });

            match result 
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(strongs_defs) = &paths.linkers
        {
            let result = Self::load_module(root, strongs_defs, |dir, name| {
                Ok(Module::Strongs(StrongsDefsModule::load(dir, name)?))
            });

            match result 
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(modules)    
        }
    }

    pub fn validate(&self) -> Result<(), Vec<PackageValidationError>>
    {
        let mut errors = vec![];

        let bibles = self.modules.values().filter_map(|m| match m {
            Module::Bible(b) => Some(b),
            _ => None,
        }).collect_vec();

        let xrefs = self.modules.values().filter_map(|m| match m {
            Module::XRef(b) => Some(b),
            _ => None,
        }).collect_vec();

        for bible in bibles
        {
            let bible_name = &bible.config.name;
            for xref in &xrefs
            {
                let xref_name = &xref.config.name;

                xref.refs.iter().enumerate().map(|(i, r)| match r {
                    XRef::Directed { source, source_text: _, targets, note: _ } => {
                        let mut ids = targets.iter().map(|t| (i, t.clone())).collect_vec();
                        ids.push((i, source.clone()));
                        ids
                    },
                    XRef::Mutual { refs, note: _ } => refs.iter().map(|r| (i, r.id.clone())).collect_vec(),
                })
                .flatten()
                .for_each(|(i, id)| {
                    if !bible.source.id_exists(&id)
                    {
                        errors.push(PackageValidationError::InvalidRefId { 
                            id: id.clone(), 
                            bible_name: bible_name.clone(), 
                            xref_name: xref_name.clone(),
                            line: i + 1,
                        });
                    }
                });
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

    fn load_module(base_dir: &str, pattern: &str, f: impl Fn(&str, &str) -> Result<Module, String>) -> Result<Vec<Module>, String>
    {
        let full_path = format!("{}/{}", base_dir, pattern);

        glob::glob(&full_path).map_err(|e| e.to_string())?.filter_map(|entry| -> Option<Result<Module, String>> {
            let entry = match entry {
                Ok(ok) => ok,
                Err(e) => return Some(Err(e.to_string())),
            };

            let path = Path::new(&entry);

            let ext = path.extension().map(|s| s.to_str()).flatten();
            if ext != Some("toml")
            {
                return None;
            }
            
            let dir = match path.parent() {
                Some(s) => s,
                None => return Some(Err(format!("Expected path {} to have a parent", path.display())))
            }.to_str().unwrap();

            let name = match path.file_stem() {
                Some(s) => s,
                None => return Some(Err(format!("Expected path {} to have a stem", path.display())))
            }.to_str().unwrap();

            Some(f(dir, name))
        }).collect()
    }
}

fn extract_ref_loc(ref_id: &RefId, verse: &Verse, r: &WordRange) -> RefId
{
    if r.start == 1 && r.end == verse.words.len() as u32 
    {
        ref_id.clone()
    } 
    else 
    {
        let (book, chapter, verse) = ref_id.get_verse_components().unwrap();

        if r.start == r.end
        {
            RefId::Single(Atom::Word { 
                book: book.into(), 
                chapter: chapter.try_into().unwrap(), 
                verse: verse.try_into().unwrap(), 
                word: r.start.try_into().unwrap() 
            })
        }
        else 
        {
            let start = Atom::Word { 
                book: book.into(), 
                chapter: chapter.try_into().unwrap(), 
                verse: verse.try_into().unwrap(), 
                word: r.start.try_into().unwrap() 
            };

            let end = Atom::Word { 
                book: book.into(), 
                chapter: chapter.try_into().unwrap(), 
                verse: verse.try_into().unwrap(), 
                word: r.end.try_into().unwrap() 
            };

            RefId::Range { from: start, to: end }
        }
    }
}
