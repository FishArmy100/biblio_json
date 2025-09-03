use std::{fs, path::Path};

use biblio_json::{PackageLoadError, Package};
use clap::{Parser, Subcommand};
use itertools::Itertools;


#[derive(Parser)]
#[command(name = "myapp", version, about = "A pretty CLI example")]
struct Cli 
{
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands
{
    Validate
    {
        path: String,
    },
    Compile
    {
        path: String,

        #[arg(short, long)]
        out: String,
    },
    Fetch
    {
        path: String,

        #[arg(short, long)]
        verse: String,

        #[arg(short, long)]
        out: Option<String>,
    }
}

fn main() 
{
    let cli = Cli::parse();
    match cli.command
    {
        Commands::Validate { path } => {
            match load_package(&path) {
                Ok(ok) => 
                {
                    println!("Package {} loaded with no errors", ok.name());
                    ok
                },
                Err(e) => return println!("Package loaded with errors {}:\n{}\n", e.len(), e.iter().join("\n"))
            };
        },
        Commands::Compile { path, out } => {
            let package = match load_package(&path) {
                Ok(ok) => 
                {
                    println!("Package {} loaded with no errors", ok.name());
                    ok
                },
                Err(e) => return println!("Package loaded with errors {}:\n{}\n", e.len(), e.iter().join("\n"))
            };

            let bin = match package.to_binary()
            {
                Ok(ok) => 
                {
                    println!("Package compiled successfully");
                    ok
                },
                Err(e) => return println!("Package compiled with error: {}", e)
            };
        },
        Commands::Fetch { path, verse, out } => todo!(),
    }
}

fn load_package(path: &str) -> Result<Package, Vec<PackageLoadError>>
{
    let p = Path::new(path);
    if p.is_dir()
    {
        Package::load(path)
    }
    else 
    {
        let bin = fs::read(path).expect("Failed to read file");
        Package::from_binary(&bin)    
    }
}
