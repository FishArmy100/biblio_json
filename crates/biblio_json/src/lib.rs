pub(crate) mod utils;
pub mod modules;
pub mod core;
pub mod html_text;
pub mod validation;
use std::{collections::HashMap, fmt::Display, num::NonZeroU32, path::Path, sync::Arc};
use flate2::Compression;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{core::{RefId, StrongsNumber, VerseId, WordRange}, html_text::HtmlText, modules::{ExternalModuleData, Module, ModuleEntry, ModuleEntryRef, ModuleValidationError, bible::{BibleModule, Verse}, commentary::CommentaryModule, dict::DictModule, strongs::{StrongsDefEntry, StrongsDefsModule, StrongsLinkEntry, StrongsLinksModule}, xrefs::XRefModule}, validation::ValidationContext};

pub const PACKAGE_FILE_NAME: &str = "biblio-json.toml";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct PackageConfig
{
    pub name: String,
    pub description: Option<HtmlText>,
    pub authors: Vec<String>,
    pub license: String,
    pub module_paths: Option<ModulePaths>,
    #[serde(default)]
    pub data: ExternalModuleData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct ModulePaths
{
    pub bibles: Option<String>,
    pub dictionaries: Option<String>,
    pub xrefs: Option<String>,
    pub strongs_defs: Option<String>,
    pub strongs_links: Option<String>,
    pub commentaries: Option<String>,
}

#[derive(Debug)]
pub enum PackageLoadError
{
    ModuleLoadingError {
        path: String,
        error: String,
    },
    ModuleValidationError {
        name: String,
        error: ModuleValidationError,
    },
    PackagePathNotDirectory(String),
    PackageConfigNotFound(String),
    PackageConfigError {
        path: String,
        error: String,
    },
    GlobError(String),
    ExpectedParent(String),
    ExpectedStem(String),
    PackagePathDoesNotExist(String),
    LoadPackageBinaryError(String),
}

impl Display for PackageLoadError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            PackageLoadError::ModuleLoadingError { path, error } => write!(f, "Error loading module '{}':\n{}", path, error),
            PackageLoadError::ModuleValidationError { name: path, error } => write!(f, "Validation error in module '{}': {}", path, error),
            PackageLoadError::PackagePathNotDirectory(path) => write!(f, "Package path '{}' is not a directory.", path),
            PackageLoadError::PackageConfigNotFound(path) => write!(f, "Package config not found at path '{}'", path),
            PackageLoadError::PackageConfigError { path, error } => write!(f, "Error loading package config '{}':\n{}", path, error),
            PackageLoadError::GlobError(error) => write!(f, "Glob error: {}", error),
            PackageLoadError::ExpectedParent(path) => write!(f, "Expected file {} to have a parent", path),
            PackageLoadError::ExpectedStem(path) => write!(f, "Expected file {} to have a stem", path),
            PackageLoadError::PackagePathDoesNotExist(path) => {
                if let Some(cwd) = std::env::current_dir().map(|p| p.display().to_string()).ok()
                {
                    write!(f, "Path for package does not exist '{}' in current directory '{}'", path, cwd) 
                }
                else 
                {
                    write!(f, "Path for package does not exist '{}'", path)    
                }
            },
            PackageLoadError::LoadPackageBinaryError(e) => write!(f, "Error when loading package binary: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchEntry
{
    pub range: WordRange,
    pub entry: ModuleEntryRef,
}

#[derive(Debug, Clone)]
pub struct VerseFetchResponse
{
    pub verse: Verse,
    pub entries: Vec<FetchEntry>,
}

#[derive(Debug)]
pub struct Package 
{
    pub config: PackageConfig,
    pub modules: HashMap<String, Module>
}

impl Package
{
    pub fn name(&self) -> &str { &self.config.name }

    pub fn load(dir_path: &str) -> Result<Self, Vec<PackageLoadError>>
    {
        let path = Path::new(dir_path);

        if !path.exists()
        {
            return Err(vec![PackageLoadError::PackagePathDoesNotExist(path.display().to_string())]);
        }

        if !path.is_dir()
        {
            return Err(vec![PackageLoadError::PackagePathNotDirectory(path.display().to_string())]);
        }

        let config_path = path.join(Path::new(PACKAGE_FILE_NAME));
        let file = utils::load_file(&config_path).map_err(|e| vec![PackageLoadError::PackageConfigNotFound(e)])?;
        
        let config = match toml::from_str::<PackageConfig>(&file) {
            Ok(ok) => ok,
            Err(e) => return Err(vec![PackageLoadError::PackageConfigError {
                path: config_path.to_str().unwrap().to_owned(),
                error: e.to_string(),
            }])
        };

        let modules = match &config.module_paths {
            Some(paths) => Self::load_modules(dir_path, paths)?,
            None => vec![]
        };

        Self::validate_modules(&modules)?;

        Ok(Self {
            config,
            modules: modules.into_iter().map(|m| (m.name().to_owned(), m)).collect()
        })
    }

    pub fn fetch(&self, verse_id: VerseId, bible: &str) -> Option<VerseFetchResponse>
    {
        let bible = self.modules.get(bible).and_then(|v| v.as_bible())?;
        let verse = bible.source.verses.get(&verse_id)?;

        let dict_entries = self.modules.values().filter_map(Module::as_dict).map(|dict| {
            verse.words.iter().enumerate().filter_map(move |(i, w)| {
                if let Some(entry) = dict.find(&w.text)
                {
                    Some(FetchEntry {
                        range: WordRange::Single((i as u32 + 1).try_into().unwrap()), // convert from 0 to 1 based indexing
                        entry: ModuleEntryRef {
                            module: dict.config.name.clone(),
                            entry_id: entry.id
                        }
                    })
                }
                else 
                {
                    None    
                }
            })
        }).flatten().collect_vec();

        let xref_entries = self.modules.values()
            .filter_map(Module::as_xrefs)
            .filter(|xrefs| xrefs.config.bible.as_ref().is_none_or(|b| *b == bible.config.name))
            .flat_map(|xrefs| {
                let entries = xrefs.entries.iter().filter(|r| r.has_verse(&verse_id)).collect_vec();
                let mut result = Vec::new();
                for entry in entries 
                {
                    // Find all word indices covered by this entry
                    let mut covered = Vec::new();
                    for (i, _) in verse.words.iter().enumerate() 
                    {
                        let word_index = (i as u32 + 1).try_into().unwrap();
                        if entry.has_verse_word(&verse_id, word_index) 
                        {
                            covered.push(word_index);
                        }
                    }

                    // Group into contiguous ranges
                    if !covered.is_empty() 
                    {
                        let mut start = covered[0];
                        let mut end = covered[0];
                        for w in covered.iter().skip(1) 
                        {
                            if w.get() == end.get() + 1 
                            {
                                end = *w;
                            } 
                            else 
                            {
                                // Push previous range
                                let range = if start == end {
                                    WordRange::Single(start)
                                } 
                                else 
                                {
                                    WordRange::Range(start, end)
                                };

                                result.push(FetchEntry {
                                    range,
                                    entry: ModuleEntryRef {
                                        module: xrefs.config.name.clone(),
                                        entry_id: entry.id(),
                                    },
                                });
                                start = *w;
                                end = *w;
                            }
                        }
                        // Push last range
                        let range = if start == end 
                        {
                            WordRange::Single(start)
                        } 
                        else 
                        {
                            WordRange::Range(start, end)
                        };

                        result.push(FetchEntry {
                            range,
                            entry: ModuleEntryRef {
                                module: xrefs.config.name.clone(),
                                entry_id: entry.id(),
                            },
                        });
                    }
                }
                result
            })
            .collect_vec();

        let commentary_entries = self.modules.values()
            .filter_map(Module::as_commentary)
            .filter(|commentary| commentary.config.bible.as_ref().is_none_or(|b| *b == bible.config.name))
            .flat_map(|commentary| {
                let entries = commentary.entries.iter().filter(|r| r.has_verse(&verse_id)).collect_vec();
                let mut result = Vec::new();
                for entry in entries 
                {
                    let mut covered = Vec::new();
                    for (i, _) in verse.words.iter().enumerate() 
                    {
                        let word_index = (i as u32 + 1).try_into().unwrap();
                        if entry.has_verse_word(&verse_id, word_index) {
                            covered.push(word_index);
                        }
                    }
                    if !covered.is_empty() 
                    {
                        let mut start = covered[0];
                        let mut end = covered[0];
                        for w in covered.iter().skip(1) 
                        {
                            if w.get() == end.get() + 1 
                            {
                                end = *w;
                            } 
                            else 
                            {
                                let range = if start == end 
                                {
                                    WordRange::Single(start)
                                } 
                                else 
                                {
                                    WordRange::Range(start, end)
                                };
                                result.push(FetchEntry {
                                    range,
                                    entry: ModuleEntryRef {
                                        module: commentary.config.name.clone(),
                                        entry_id: entry.id,
                                    },
                                });
                                start = *w;
                                end = *w;
                            }
                        }
                        let range = if start == end {
                            WordRange::Single(start)
                        } else {
                            WordRange::Range(start, end)
                        };
                        result.push(FetchEntry {
                            range,
                            entry: ModuleEntryRef {
                                module: commentary.config.name.clone(),
                                entry_id: entry.id,
                            },
                        });
                    }
                }
                result
            })
            .collect_vec();

        let strongs = self.modules.values()
            .filter_map(Module::as_strongs_links)
            .find(|links| links.config.bible == bible.config.name)
            .map(|links| links.get_links(&verse_id).map(|l| ModuleEntryRef { 
                module: links.config.name.clone(),
                entry_id: l.id,
            }))
            .flatten();
            
        let mut entries = dict_entries.into_iter()
            .chain(xref_entries.into_iter())
            .chain(commentary_entries.into_iter())
            .collect_vec();

        if let Some(strongs) = strongs
        {
            entries.push(FetchEntry { 
                range: WordRange::Range(NonZeroU32::MIN, (verse.words.len() as u32).try_into().unwrap()), 
                entry: strongs
            });
        }

        Some(VerseFetchResponse { 
            verse: verse.clone(), 
            entries
        })
    }

    pub fn fetch_strongs(&self, strongs: &StrongsNumber) -> Vec<&StrongsDefEntry>
    {
        self.modules.values()
            .filter_map(|m| match m {
                Module::StrongsDefs(d) => Some(d.as_ref()),
                _ => None
            })
            .filter_map(|defs| defs.get_def(strongs))
            .collect()
    }

    pub fn fetch_entry<'a>(&'a self, entry_ref: ModuleEntryRef) -> Option<ModuleEntry<'a>>
    {
        let module = self.get_mod(&entry_ref.module)?;
        let module_entry = match module 
        {
            Module::Bible(bible) => ModuleEntry::Verse(bible.source.verses.values().find(|v| v.id == entry_ref.entry_id)?),
            Module::Dictionary(dict) => ModuleEntry::Dictionary(dict.entries.iter().find(|e| e.id == entry_ref.entry_id)?),
            Module::XRef(xref) => ModuleEntry::XRef(xref.entries.iter().find(|e| e.id() == entry_ref.entry_id)?),
            Module::StrongsDefs(strongs_defs) => ModuleEntry::StrongsDef(strongs_defs.entries.iter().find(|e| e.id == entry_ref.entry_id)?),
            Module::StrongsLinks(strongs_links) => ModuleEntry::StrongsLink(strongs_links.entries.iter().find(|e| e.id == entry_ref.entry_id)?),
            Module::Commentary(commentary) => ModuleEntry::Commentary(commentary.entries.iter().find(|e| e.id == entry_ref.entry_id)?),
        };

        Some(module_entry)
    }

    pub fn to_binary(&self) -> Result<Vec<u8>, String>
    {
        let bin = PackageBinary {
            config: self.config.clone(),
            modules: self.modules.values().cloned().collect_vec(),
        };

        let uncompressed = bincode::serde::encode_to_vec(bin, bincode::config::standard())
            .map_err(|e| e.to_string())?;

        Ok(utils::compress(&uncompressed, Compression::best()))
    }

    pub fn load_binary<P>(path: &str) -> Result<Self, Vec<PackageLoadError>>
    {
        let bin = utils::load_file_bin(path).map_err(|e| vec![PackageLoadError::LoadPackageBinaryError(e)])?;
        Self::from_binary(&bin)
    }

    pub fn from_binary(bin: &[u8]) -> Result<Self, Vec<PackageLoadError>>
    {
        let uncompressed = utils::decompress(bin);
        let (bin, _): (PackageBinary, usize) = bincode::serde::decode_from_slice(&uncompressed, bincode::config::standard())
            .map_err(|e| vec![PackageLoadError::LoadPackageBinaryError(e.to_string())])?;

        Self::validate_modules(&bin.modules)?;

        Ok(Self {
            modules: bin.modules.into_iter().map(|m| (m.name().to_owned(), m)).collect(),
            config: bin.config,
        })
    }

    pub fn get_mod(&self, name: &str) -> Option<&Module>
    {
        self.modules.get(name)
    }

    fn validate_modules(modules: &Vec<Module>) -> Result<(), Vec<PackageLoadError>>
    {
        let bibles = modules.iter().filter_map(|m| match m {
            Module::Bible(b) => Some(b.clone()),
            _ => None,
        }).map(|b| (b.config.name.clone(), b)).collect::<HashMap<_, _>>();

        let context = ValidationContext {
            bibles: &bibles
        };

        let mut errors = vec![];
        for m in modules
        {
            if let Err(errs) = m.validate(&context)
            {
                errors.extend(errs.into_iter().map(|error| PackageLoadError::ModuleValidationError { 
                    name: m.name().to_string(), 
                    error,
                }));
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

    fn load_modules(root: &str, paths: &ModulePaths) -> Result<Vec<Module>, Vec<PackageLoadError>>
    {
        let mut modules = vec![];
        let mut errors = vec![];
        
        if let Some(bibles_path) = &paths.bibles
        {
            let result = Self::load_module(root, &bibles_path, |dir, name| 
            {
                Ok(Module::Bible(Arc::new(BibleModule::load_json(dir, name)?)))
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
                Ok(Module::Dictionary(Arc::new(DictModule::load_json(dir, name)?)))
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
                Ok(Module::XRef(Arc::new(XRefModule::load_json(dir, name)?)))
            });

            match result
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(strongs_defs) = &paths.strongs_defs
        {
            let result = Self::load_module(root, strongs_defs, |dir, name| {
                Ok(Module::StrongsDefs(Arc::new(StrongsDefsModule::load_json(dir, name)?)))
            });

            match result 
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(strongs_links) = &paths.strongs_links
        {
            let result = Self::load_module(root, strongs_links, |dir, name| {
                Ok(Module::StrongsLinks(Arc::new(StrongsLinksModule::load_json(dir, name)?)))
            });

            match result 
            {
                Ok(ok) => modules.extend(ok),
                Err(e) => errors.push(e),
            }
        }

        if let Some(commentaries) = &paths.commentaries
        {
            let result = Self::load_module(root, commentaries, |dir, name| {
                Ok(Module::Commentary(Arc::new(CommentaryModule::load_json(dir, name)?)))
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

    fn load_module(base_dir: &str, pattern: &str, f: impl Fn(&str, &str) -> Result<Module, String>) -> Result<Vec<Module>, PackageLoadError>
    {
        let full_path = format!("{}/{}", base_dir, pattern);

        let paths = glob::glob(&full_path).map_err(|e| PackageLoadError::GlobError(e.to_string()))?;
        paths.filter_map(|entry| -> Option<Result<Module, PackageLoadError>> {
            let entry = match entry {
                Ok(ok) => ok,
                Err(e) => return Some(Err(PackageLoadError::GlobError(e.to_string()))),
            };

            let path = Path::new(&entry);

            let ext = path.extension().map(|s| s.to_str()).flatten();
            if ext != Some("toml")
            {
                return None;
            }
            
            let dir = match path.parent() {
                Some(s) => s,
                None => return Some(Err(PackageLoadError::ExpectedParent(path.display().to_string())))
            }.to_str().unwrap();

            let name = match path.file_stem() {
                Some(s) => s,
                None => return Some(Err(PackageLoadError::ExpectedStem(path.display().to_string())))
            }.to_str().unwrap();

            Some(f(dir, name).map_err(|e| PackageLoadError::ModuleLoadingError { 
                path: path.canonicalize().ok().map(|p| p.display().to_string()).unwrap(), 
                error: e
            }))
        }).collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageBinary
{
    pub modules: Vec<Module>,
    pub config: PackageConfig,
}