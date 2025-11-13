use std::str::FromStr;

use biblio_json::{Package, core::VerseId, modules::readings::date::{ReadingsDate, ReadingsMonth}};
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

    println!("{}", package.modules.values().map(|m| m.name()).join("\n"));

    let robert_roberts = package.get_mod("Bible in One Year Readings").unwrap().as_readings().unwrap();
    let current_date = ReadingsDate::now();
    let start_date = ReadingsDate::new(current_date.year(), ReadingsMonth::January, 1).unwrap();
    let current = robert_roberts.get_reading(start_date, current_date).unwrap();

    println!("{:#?}", current);
    
    // let verse = VerseId::from_str("Gen.2.10").unwrap();
    // let fetch = package.fetch(verse, "KJV");
    // println!("{:#?}", fetch);

    // let bible = package.get_mod("KJV").unwrap().as_bible().unwrap();
    // let abbrev = bible.get_abbreviated_book(OsisBook::John);

    // println!("{:#?}", abbrev);

    // let notebook = package.get_mod("Test Notebook").unwrap().as_notebook().unwrap();
    // let entry = notebook.entries.iter().find(|e| e.id() == 0).unwrap();
    // println!("{:#?}", entry);
}

