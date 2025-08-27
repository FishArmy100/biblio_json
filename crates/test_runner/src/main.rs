use biblio_json::{html_text as html_text, Package};
use itertools::Itertools;

fn main()
{
    // let package = match Package::load("./res") {
    //     Ok(ok) => {
    //         println!("Package loaded!");
    //         ok
    //     },
    //     Err(e) => return println!("Package loaded with errors:\n{}\n", e.iter().join("\n"))
    // };

    let text = "
    <h1>Hello World!</h1>
    <p>
    This is a <b>test</b> message <img src=\"<bbb>\">
    </p>
    ";

    let html = html_text::HtmlText::from_str(text);
    println!("{:#?}", html.unwrap());
}

