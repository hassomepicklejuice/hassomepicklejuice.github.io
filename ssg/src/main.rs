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

#[derive(Clone, Copy, Debug)]
struct FileHandle<'a> {
    file: &'a Path,
    in_dir: &'a Path,
    out_dir: &'a Path,
}

impl<'a> FileHandle<'a> {
    fn out_file(&self) -> PathBuf {
        self.out_dir.join(self.file)
    }
    fn in_file(&self) -> PathBuf {
        self.in_dir.join(self.file)
    }
    fn copy(&self) -> Result<()> {
        let in_file = self.in_file();
        let out_file = self.out_file();
        if in_file.is_file() {
            fs::copy(in_file, out_file)?;
        } else if in_file.is_dir() {
            fs::create_dir_all(out_file)?;
            for entry in fs::read_dir(in_file)? {
                let path = entry?.path();
                let file = FileHandle {
                    file: &path,
                    ..*self
                };
                file.copy()?;
            }
        }
        Ok(())
    }
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

    let root = FileHandle {
        file: Path::new(""),
        in_dir: &args.in_dir,
        out_dir: &args.out_dir,
    };

    render_dir(&mut handlebars, root)?;

    let default_style = FileHandle {
        file: &root.file.join("style.css"),
        ..root
    };

    default_style.copy()?;

    Ok(())
}

fn render_dir(hb: &mut Handlebars, dir: FileHandle) -> Result<()> {
    if !dir.in_dir.is_dir() {
        bail!(
            "Input path should be a directory, {} is not a directory",
            dir.in_dir.display()
        )
    }

    for entry in WalkDir::new(dir.in_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file = FileHandle {
            file: entry.path().strip_prefix(dir.in_dir)?,
            ..dir
        };
        if entry.file_type().is_dir() {
            fs::create_dir_all(file.out_file())?;
        } else if entry.file_type().is_file() {
            if let Err(_) = render_file(hb, file) {
                continue;
            }
        }
    }

    Ok(())
}

fn render_file(hb: &mut Handlebars, file: FileHandle) -> Result<()> {
    let mut data = read_source(file.in_file()).context("Failed to read source file")?;

    parse_body(&mut data)?;

    let template = data["template"]
        .as_str()
        .context("'template' field should be a String")?;
    let rendered = hb
        .render(template, &data)
        .with_context(|| format!("Failed to render template {template} with data {data:#?}"))?;

    fs::write(file.out_file(), rendered).context("Failed to write rendered output to file")?;

    if let Some(stylesheet) = data.get("stylesheet").and_then(|v| v.as_str()) {
        let stylesheet = FileHandle {
            file: Path::new(stylesheet),
            ..file
        };
        stylesheet.copy()?;
    }

    if let Some(script) = data.get("script").and_then(|v| v.as_str()) {
        let script = FileHandle {
            file: Path::new(script),
            ..file
        };
        script.copy()?;
    }

    match data.get("assets") {
        None => {}
        Some(Value::String(asset)) => {
            let asset = FileHandle {
                file: Path::new(asset),
                ..file
            };
            asset.copy()?;
        }
        Some(Value::Array(assets)) => {
            for asset in assets.into_iter().filter_map(|v| v.as_str()) {
                let asset = FileHandle {
                    file: Path::new(asset),
                    ..file
                };
                asset.copy()?;
            }
        }
        _ => bail!("the 'assets' field should be a single file or an array of file"),
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
    let (meta, body) = content
        .split_once("*** ssg ***\n")
        .context("Not a source file")?;
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
