use std::{fs, path::Path, str::FromStr, time::SystemTime};

use biblio_json::{modules::{xrefs::XRef, Module}, ref_id::RefId, Package};
use itertools::Itertools;

fn main()
{
    let package = match Package::load("./res") {
        Ok(ok) => ok,
        Err(e) => return println!("Errors:\n{}\n", e.iter().join("\n"))
    };

    let Some(Module::Bible(bible)) =  package.modules.iter().find(|m| m.is_bible()) else {
        panic!("No bible module found");
    };

    if let Some(Module::XRef(xrefs)) = package.modules.iter().find(|m| m.is_xrefs())
    {
        let all_ids = xrefs.refs.iter().map(|r| match r {
            XRef::Directed { source, source_text: _, targets, note: _ } => {
                let mut ids = targets.clone();
                ids.push(source.clone());
                ids
            },
            XRef::Mutual { refs, note: _ } => refs.iter().map(|r| r.id.clone()).collect_vec(),
        }).flatten().collect_vec();

        let mut count: u64 = 0;

        for id in all_ids
        {
            if !bible.source.id_exists(&id)
            {
                panic!("Id `{}` does not exist in the bible", id);
            }
            else 
            {
                count += 1;    
            }
        }

        println!("Everything is fine! {}", count);
    }
    
    // let Some(Module::Bible(kjv)) = package.modules.get(0) else {
    //     return;
    // };

    // println!("{} has {} books.", kjv.name, kjv.source.book_infos.len())
}

