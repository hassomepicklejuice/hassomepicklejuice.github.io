use std::fs;

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
        let content = fs::read_to_string(source).expect("Reading source file");
        let (meta, body) = content.split_once("***\n").unwrap_or(("", &content));
        let mut meta = meta.parse::<Table>().unwrap_or_default();
        meta.insert("BODY".to_string(), body.into());

        let template = meta
            .get("template")
            .map(|v| v.to_string())
            .unwrap_or("article".to_string());

        let rendered = handlebars
            .render(&template, &meta)
            .expect("Rendering source");
        eprintln!("{rendered}");
    }

    println!("Hello, world!");
}
