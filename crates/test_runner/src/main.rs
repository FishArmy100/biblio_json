use std::{fs, str::FromStr};

use biblio_json::{Package, core::{OsisBook, VerseId}};
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
    
    // let verse = VerseId::from_str("Gen.2.10").unwrap();
    // let fetch = package.fetch(verse, "KJV");

    // let bible = package.get_mod("KJV").unwrap().as_bible().unwrap();
    // let abbrev = bible.get_abbreviated_book(OsisBook::John);

    // println!("{:#?}", abbrev);

    let notebook = package.get_mod("Test Notebook").unwrap().as_notebook().unwrap();
    let entry = notebook.entries.iter().find(|e| e.id() == 0).unwrap();
    println!("{:#?}", entry);
}

