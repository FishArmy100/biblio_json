use std::{fs, path::Path};

use flate2::{Compression, read::{ZlibDecoder, ZlibEncoder}};
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use std::io::{Read, Write};

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

pub fn load_json<T, P>(path: P) -> Result<T, String> 
    where P : AsRef<Path>,
          T : for<'a> Deserialize<'a>
{
    let src = load_file(path)?;
    serde_json::from_str(&src)
        .map_err(|e| e.to_string())
}

pub fn load_json_lines<T, P>(path: P) -> Result<Vec<(T, usize)>, String>
    where P : AsRef<Path>,
          T : for<'a> Deserialize<'a> + Send + Sync + 'static
{
    let src = load_file(path)?;
    src.lines().enumerate().filter(|(_, v)| !v.is_empty()).collect_vec().into_par_iter().map(|(line, json)| {
        match serde_json::from_str::<T>(json)
        {
            Ok(ok) => Ok((ok, line)),
            Err(e) => Err(e.to_string())
        }
    }).collect()
}

pub fn write_file<P>(path: P, src: &str) -> Result<(), String>
    where P : AsRef<Path>
{
    fs::write(path, src).map_err(|e| e.to_string())
}

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