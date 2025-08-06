use std::{fs, path::Path};

use biblio_json::{modules::Module, Package};
use itertools::Itertools;

fn main()
{
    let package = match Package::load("./res") {
        Ok(ok) => ok,
        Err(e) => return println!("Errors:\n{}\n", e.iter().join("\n"))
    };

    if let Some(Module::Dictionary(dict)) = package.modules.iter().find(|m| m.is_dict())
    {
        let name = "Zuzims'";
        if let Some(entry) = dict.find(name)
        {
            println!("{}: {}", entry.term, entry.definitions.iter().join(", "))
        }
    }
    
    // let Some(Module::Bible(kjv)) = package.modules.get(0) else {
    //     return;
    // };

    // println!("{} has {} books.", kjv.name, kjv.source.book_infos.len())
}

