use std::str::FromStr;

use biblio_json::{html_text, Package, PackageValidationError};
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

    let text = "
    <h1>Hello World!</h1>
    <p>
    This is a <b>test</b> message
    </p>
    ";

    let html = html_text::parse(text);
    println!("{:#?}", html.unwrap().to_html());
}

