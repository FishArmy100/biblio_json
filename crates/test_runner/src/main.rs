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
    
    if let Some(Module::Bible(bible)) = package.get_mod("KJV")
    {
        let desc = &bible.config.description;
        println!("\n\n{}\n\n", desc.as_ref().unwrap().to_string())
    }
}

