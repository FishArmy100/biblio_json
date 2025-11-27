use std::{fs, hash::Hash, path::Path};

use flate2::{Compression, read::{ZlibDecoder, ZlibEncoder}};
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use std::io::Read;

pub fn load_file<P>(path: P) -> Result<String, String>
    where P : AsRef<Path>
{
    match fs::read(path)
    {
        Ok(ok) => match String::from_utf8(ok)
        {
            Ok(ok) => Ok(ok),
            Err(err) => return Err(err.to_string()),
        }
        Err(err) => return Err(err.to_string())
    }
}

pub fn load_file_bin<P>(path: P) -> Result<Vec<u8>, String>
    where P : AsRef<Path>
{
    fs::read(path).map_err(|e| e.to_string())
}

pub fn load_toml<T, P>(path: P) -> Result<T, String>
    where P : AsRef<Path>,
          T : for<'a> Deserialize<'a>
{
    let src = load_file(path)?;
    toml::from_str(&src)
        .map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn load_json<T, P>(path: P) -> Result<T, String> 
    where P : AsRef<Path>,
          T : for<'a> Deserialize<'a>
{
    let src = load_file(path)?;
    serde_json::from_str(&src)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub struct LoadJsonLinesErr
{
    pub line: usize,
    pub error: String,
}

pub struct JsonLine<T> where T : for<'de> Deserialize<'de>
{
    pub value: T,
    pub line: usize,
}

pub enum LoadJsonLinesResult<T> 
    where T : for<'de> Deserialize<'de>
{
    Ok(Vec<JsonLine<T>>),
    LoadFailed(String),
    ParseFailed(Vec<LoadJsonLinesErr>),
}

impl<T> LoadJsonLinesResult<T> 
    where T : for<'de> Deserialize<'de>
{
    pub fn stringify_error(self) -> Result<Vec<JsonLine<T>>, String>
    {
        match self 
        {
            LoadJsonLinesResult::Ok(ok) => Ok(ok),
            LoadJsonLinesResult::LoadFailed(e) => Err(e),
            LoadJsonLinesResult::ParseFailed(errs) => {
                let message = errs.iter()
                    .enumerate()
                    .map(|(i, e)| format!("Error {}: line {}\n{}", i + 1, e.line, e.error))
                    .join("\n\n");

                Err(message)
            },
        }
    }
}

pub fn load_json_lines<T, P>(path: P) -> LoadJsonLinesResult<T>
    where P : AsRef<Path>,
          T : for<'a> Deserialize<'a> + Send + Sync + 'static
{
    let src = match load_file(path)
    {
        Ok(ok) => ok,
        Err(e) => return LoadJsonLinesResult::LoadFailed(e)
    };

    let result = src.lines().enumerate().filter(|(_, v)| !v.is_empty()).collect_vec().into_par_iter().map(|(line, json)| {
        match serde_json::from_str::<T>(json)
        {
            Ok(ok) => Ok(JsonLine { value: ok, line: line + 1 }),
            Err(e) => Err(LoadJsonLinesErr { error: e.to_string(), line: line + 1 })
        }
    }).fold(|| (vec![], vec![]), |mut id, value| {
        match value
        {
            Ok(ok) => id.0.push(ok),
            Err(err) => id.1.push(err),
        };

        id
    }).reduce(
        || (Vec::new(), Vec::new()),
        |mut a, mut b| {
            a.0.append(&mut b.0);
            a.1.append(&mut b.1);
            a
        },
    );

    if result.1.len() > 0
    {
        LoadJsonLinesResult::ParseFailed(result.1)
    }
    else 
    {
        LoadJsonLinesResult::Ok(result.0)
    }
}

#[allow(dead_code)]
pub fn write_file<P>(path: P, src: &str) -> Result<(), String>
    where P : AsRef<Path>
{
    fs::write(path, src).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn write_file_bin<P>(path: P, src: &[u8]) -> Result<(), String>
    where P : AsRef<Path>
{
    fs::write(path, src).map_err(|e| e.to_string())
}

pub fn compress(data: &[u8], compression: Compression) -> Vec<u8> 
{
    let mut encoder = ZlibEncoder::new(data, compression);
    let mut compressed = Vec::new();
    encoder.read_to_end(&mut compressed).unwrap();
    compressed
}

pub fn decompress(data: &[u8]) -> Vec<u8> 
{
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

pub fn find_duplicates<I, T>(i: I) -> impl Iterator<Item = T>
    where I : Iterator<Item = T>,
          T : Hash + Eq,
{
    i.counts().into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(val, _)| val)
}