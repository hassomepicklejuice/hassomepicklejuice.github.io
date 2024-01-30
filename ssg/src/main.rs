use std::{fs, path::Path};

use handlebars::Handlebars;
use toml::Table;

fn main() {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory("templates", Default::default())
        .expect("Registering templates");

    let mut sources = std::env::args();
    sources.next();

    for source in sources {
        let meta = read_source(source).expect("Reading source");

        let template = meta["template"].as_str().expect("'template' is a string");
        let typ = meta["type"].as_str().expect("'type' is a string");

        match typ {
            "html" => {
                let rendered = handlebars
                    .render(template, &meta)
                    .expect("Rendering source");
                eprintln!("{rendered}");
            }
            _ => {
                eprintln!("Cannot handle files of type {typ} yet");
            }
        }
    }

    println!("Hello, world!");
}

fn read_source(source: impl AsRef<Path>) -> Result<Table, std::io::Error> {
    let content = fs::read_to_string(source)?;
    let (meta, body) = content.split_once("***\n").unwrap_or(("", &content));
    let mut meta = meta.parse::<Table>().unwrap_or_default();

    meta.entry("template").or_insert("article".into());
    meta.entry("type").or_insert("html".into());
    meta.insert("BODY".to_string(), body.into());

    Ok(meta)
}
