use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
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
    #[arg(short, long, default_value_os_t = PathBuf::from("."))]
    out_dir: PathBuf,
    /// Paths to the files to be rendered, or directories containing the source files
    #[arg(required = true, num_args = 1..)]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = std::env::current_dir().context("Failed to get the current working directory")?;

    let mut handlebars = Handlebars::new();

    for template in args.templates {
        if template.is_file() {
            let name = match template.file_stem().and_then(|name| name.to_str()) {
                Some(name) => name,
                _ => continue,
            };
            handlebars
                .register_template_file(name, &template)
                .with_context(|| {
                    format!(
                        "Failed to register the template file at {}",
                        template.display(),
                    )
                })?;
        } else if template.is_dir() {
            handlebars
                .register_templates_directory(&template, Default::default())
                .with_context(|| {
                    format!(
                        "Failed to register the template files in {}",
                        template.display()
                    )
                })?;
        }
    }

    for file in args.files {
        let mut data = read_source(&file)
            .with_context(|| format!("Failed to read source file at {}", file.display()))?;

        parse_body(&mut data)?;

        let template = data["template"]
            .as_str()
            .context("'template' field should be a String")?;

        let rendered = handlebars
            .render(template, &data)
            .with_context(|| format!("Failed to render data {data:?} to template {template}"))?;

        let out_file = args.out_dir.join(match file.strip_prefix(&cwd) {
            Ok(file) => file.to_owned(),
            Err(_) if file.has_root() => file.file_name().context("Not a valid filename")?.into(),
            Err(_) => file,
        });

        fs::create_dir_all(
            out_file
                .parent()
                .context("Output file has no parent directory")?,
        )
        .with_context(|| {
            format!(
                "Failed to create parent directory of output file {}",
                out_file.display()
            )
        })?;

        fs::write(&out_file, rendered)
            .with_context(|| format!("Failed to write output file {}", out_file.display()))?;
    }

    Ok(())
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
