use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use clap::Parser;
use handlebars::Handlebars;
use toml::{Table, Value};

/// Custom static site generator.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to Handlebars template files or directories containing template files
    #[arg(short, long)]
    templates: Vec<PathBuf>,
    /// Path to the output directory, defaults to the current directory
    #[arg(short, long)]
    out_dir: Option<PathBuf>,
    /// Paths to the files to be rendered, or directories containing the source files
    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    assert!(!args.files.is_empty());

    fs::create_dir_all(args.out_dir.unwrap_or_else(|| PathBuf::from(".")))
        .expect("Created output directory");

    let mut handlebars = Handlebars::new();

    for template in args.templates {
        if template.is_file() {
            let name = match template.file_stem().and_then(|name| name.to_str()) {
                Some(name) => name,
                _ => continue,
            };
            handlebars
                .register_template_file(name, &template)
                .expect("Registering template file");
        } else if template.is_dir() {
            handlebars
                .register_templates_directory(template, Default::default())
                .expect("Registering template directory");
        }
    }

    for file in args.files {
        let mut data = read_source(file).expect("Reading source");

        parse_body(&mut data).expect("Body was parsed succesfully");

        let template = data["template"]
            .as_str()
            .expect("'template' should be a string");

        let rendered = handlebars
            .render(template, &data)
            .expect("Rendering source");
    }

    println!("Hello, world!");
}

fn parse_body(data: &mut Table) -> Result<()> {
    match data["type"] {
        Value::String(ref typ) if typ == "html" => Ok(()),
        Value::String(ref typ) => Err(anyhow!("Cannot handle files of type {typ} yet")),
        ref x => Err(anyhow!(
            "Expected the 'type' field to be a String. Instead it was {x:?}"
        )),
    }
}

fn read_source(source: impl AsRef<Path>) -> Result<Table> {
    let content = fs::read_to_string(source.as_ref())?;
    let (meta, body) = content.split_once("***\n").unwrap_or(("", &content));
    let mut data = meta.parse::<Table>().unwrap_or_default();

    data.entry("template").or_insert("article".into());

    data.entry("type").or_insert(
        source
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(anyhow!("No extension present"))?
            .into(),
    );

    data.insert("BODY".to_string(), body.into());

    Ok(data)
}
