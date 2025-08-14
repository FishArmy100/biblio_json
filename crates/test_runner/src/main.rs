use std::str::FromStr;

use biblio_json::{ref_id::RefId, Package, PackageValidationError};
use itertools::Itertools;

fn main()
{
    let package = match Package::load("./res") {
        Ok(ok) => {
            println!("Package loaded!");
            ok
        },
        Err(e) => return println!("Package loaded with errors:\n{}\n", e.iter().join("\n"))
    };
    
    if let Err(errors) = package.validate()
    {
        let msg = errors.into_iter()
        .filter_map(|e| match e {
            PackageValidationError::InvalidRefId { id, bible_name: _, xref_name: _, line } => Some((line, id)),
        })
        .unique_by(|(_, id)| id.clone())
        .enumerate()
        .map(|(i, (line, id))| format!(" {}. {} on {}", i + 1, id, line))
        .join("\n");

        println!("Validation errors:\n{}", msg)
    }
    else 
    {
        println!("Validation passed!");
    }

    let fetch_ref = RefId::from_str("Gen.2.8").unwrap();
    let stuff = package.fetch("kjv", &fetch_ref);
    println!("{:#?}", stuff);
}

