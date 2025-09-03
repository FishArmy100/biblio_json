use std::{fs::{self, File}, io::Write, path::Path, str::FromStr, time::SystemTime};

use biblio_json::{Package, PackageLoadError, core::VerseId};
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
    /// Validate a package at the given path (directory or binary file)
    Validate
    {
        /// Path to the package (directory or binary file)
        path: String,
    },
    /// Compile a package from the given path and write the binary output
    Compile
    {
        /// Path to the package (directory)
        path: String,

        /// Output file path for the compiled binary
        #[arg(short, long)]
        out: String,
    },
    /// Fetch a verse from a compiled package
    Fetch
    {
        /// Path to the compiled package (binary file)
        path: String,

        /// Verse ID to fetch (e.g. "GEN.1.1")
        #[arg(short, long)]
        verse: String,

        /// Bible identifier to use
        #[arg(short, long)]
        bible: String,

        /// Optional output file path (if not set, prints to stdout)
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

            match write_to_file(&out, &bin)
            {
                Ok(_) => {},
                Err(e) => println!("Error when writing to file '{}': {}", out, e)
            }
        },
        Commands::Fetch { path, verse, bible, out } => {
            let package = match load_package(&path) {
                Ok(ok) => 
                {
                    println!("Package {} loaded with no errors", ok.name());
                    ok
                },
                Err(e) => return println!("Package loaded with errors {}:\n{}\n", e.len(), e.iter().join("\n"))
            };

            let verse_id = match VerseId::from_str(&verse)
            {
                Ok(ok) => ok,
                Err(_) => return println!("Verse id {} is not in the proper format", verse)
            };

            match package.fetch(verse_id, &bible)
            {
                Some(result) => {
                    let result = format!("{:#?}", result);
                    if let Some(out) = out 
                    {
                        if let Err(e) = write_to_file(&out, result.as_bytes())
                        {
                            return println!("Error when writing to file '{}': {}", out, e);
                        }
                        println!("Fetch successful! Wrote output to '{}'", out);
                    }
                    else
                    {
                        println!("Fetched:\n{}", result)
                    }
                },
                None => println!("No result for fetching verse {} in bible {}", verse_id, bible)
            };
        },
    }
}

fn load_package(path: &str) -> Result<Package, Vec<PackageLoadError>>
{
    let p = Path::new(path);
    let current = SystemTime::now();
    let pkg = if p.is_dir()
    {
        Package::load(path)
    }
    else 
    {
        let bin = fs::read(path).expect("Failed to read file");
        Package::from_binary(&bin)    
    };

    let elapsed = current.elapsed().unwrap().as_secs_f32();
    println!("Loaded package in {}ms", elapsed * 1000.0);
    pkg
}

fn write_to_file<P>(path: P, content: &[u8]) -> Result<(), String> 
    where P : AsRef<Path>
{
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Open file (creates if not exists, truncates if exists)
    let mut file = File::create(path).map_err(|e| e.to_string())?;

    // Write content
    file.write_all(content).map_err(|e| e.to_string())?;

    Ok(())
}
