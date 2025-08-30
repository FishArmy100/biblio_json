use std::fs;

use biblio_json::{modules::Module, Package};
use itertools::Itertools;

fn main()
{
    let package = match Package::load("./res") {
        Ok(ok) => 
        {
            println!("Package loaded!");
            ok
        },
        Err(e) => return println!("Package loaded with errors:\n{}\n", e.iter().join("\n"))
    };

    let bin = package.to_binary().unwrap();
    let package = match Package::from_binary(&bin) {
        Ok(ok) => ok,
        Err(e) => panic!("Errors when loading package:\n{}", e.iter().map(|e| e.to_string()).join("\n"))
    };

    if let Some(bible) = package.modules.values().find_map(|m| m.as_bible())
    {
        let verse_id = "Prov.3.5".parse().unwrap();
        let verse = bible.source.verses.get(&verse_id).unwrap();
        println!("[{} {}]: {}", verse_id, bible.config.name, verse.words.iter().map(|w| &w.text).join(" "));
    }

    println!("Package serializing/deserializing working!");
}

