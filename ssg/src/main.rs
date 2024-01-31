use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use handlebars::Handlebars;
use toml::{Table, Value};
use walkdir::WalkDir;

/// Custom static site generator.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to Handlebars template files or directories containing template files
    #[arg(short, long, default_values_os_t = [PathBuf::from("templates")])]
    templates: Vec<PathBuf>,
    /// Path to the output directory
    #[arg(short, long, default_value_os_t = PathBuf::from("docs"))]
    out_dir: PathBuf,
    /// Path to the input directory
    #[arg(short, long, default_value_os_t = PathBuf::from("src"))]
    in_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

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

    render_dir(&mut handlebars, &args.in_dir, &args.out_dir)?;

    Ok(())
}

fn render_dir(hb: &mut Handlebars, in_dir: &Path, out_dir: &Path) -> Result<()> {
    if !in_dir.is_dir() {
        bail!(
            "Input path should be a directory, {} is not a directory",
            in_dir.display()
        )
    }

    for entry in WalkDir::new(in_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let in_file = entry.path();
        let out_file = out_dir.join(in_file.strip_prefix(in_dir)?);
        if in_file.is_dir() {
            fs::create_dir_all(out_file)?;
        } else if in_file.is_file() {
            render_file(hb, in_file, &out_file)?;
        }
    }

    Ok(())
}

fn render_file(hb: &mut Handlebars, in_file: &Path, out_file: &Path) -> Result<()> {
    let mut data = read_source(in_file).context("Failed to read source file")?;

    parse_body(&mut data)?;

    let template = data["template"]
        .as_str()
        .context("'template' field should be a String")?;
    let rendered = hb
        .render(template, &data)
        .with_context(|| format!("Failed to render template {template} with data {data:#?}"))?;

    fs::write(out_file, rendered).context("Failed to write rendered output to file")?;
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
