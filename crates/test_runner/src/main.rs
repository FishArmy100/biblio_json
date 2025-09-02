use std::{fs, str::FromStr};

use biblio_json::{Package, core::VerseId, modules::Module};
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
    
    let verse = VerseId::from_str("Gen.2.10").unwrap();
    let fetch = package.fetch(verse, "KJV");
    println!("{:#?}", fetch);
}

