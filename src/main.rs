use std::{env, fs};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::process::exit;
use clap::{App, AppSettings, Arg};
use askama::Template;
use serde::{Deserialize, Serialize};
use rustyline::Editor;
use anyhow::Result;
use ini::Ini;
use tera::Context;
use toml::Value;
use templates::ReadmeTemplate;
use crate::error::{Error, err};


static VERSION: &str = env!("CARGO_PKG_VERSION");
static BIN_NAME: &str = env!("CARGO_BIN_NAME");
static TEMPLATE_DIR: Dir<'_> = include_dir!("templates");

mod context;
mod error;


pub struct WritingOptions {
    pub force: bool,
}


struct FileTemplate {
    relative_path: String,
    template_text: String,

}


struct CompletedFile {
    final_path: PathBuf,
    rendered: String,
}



fn find_templates(template_dir: Dir) -> Vec<FileTemplate> {
    let mut templates = Vec::new();
    for entry in template_dir.entries() {
        match entry {
            Dir(d) => {
                let mut subdir_templates = write_directory(d, context);
                templates.append(&mut subdir_templates);
            }
            File(f) => {
                templates.push(FileTemplate {
                    relative_path: f.path().to_str().unwrap().to_string(),
                    template_text: f.contents_to_string().unwrap(),
                });
            }
        }
    }
    templates
}


fn write_templates(template_dir: Dir, context: Context, output_path: &str, options: &WritingOptions) -> Result<(), Error> {
    let file_templates = find_templates(&template_dir);

    let to_dir = output_path.ends_with(MAIN_SEPARATOR.to_string());
    if file_templates.len() > 1 && (output_path == "-" || !to_dir) {
        exit_with!("Error: You are trying to write multiple file templates, but the output is not a directory. Got: {}", output_path);
    }
    let output_path = Path::new(output_path);

    let mut final_files = Vec::new();
    for template in file_templates {
        final_files.push(CompletedFile {
            final_path: if to_dir { output_path.join(template.relative_path).to_owned() } else { output_path.to_owned() },
            rendered: template.template_text.render_str(&context)?,
        });
    }

    if output_path == "-" {
        for file in final_files {
            std::io::stdout()
                .write_all(file.rendered.as_bytes()).unwrap();
            return Ok(());
        }
    }

    for file in final_files {
        if file.final_path.is_file() && !options.force {
            return err!("{}: File already exists.", file.final_path.display());
        }
    }

    for file in final_files {
        fs::create_dir_all(file.final_path.parent().unwrap()).unwrap();
        File::create(file.final_path)
            .map_err(|_| err!("{}: Failed to create file.", path.display()))?
            .write_all(text.as_bytes())
            .map_err(|_| err!("{}: Failed to write to file.", path.display()))?;
        eprintln!("{}: Wrote file.", file.final_path.display());
    }
    Ok(())
}


fn resolve_template_variables(vec: Vec<&str>) -> Result<Context, error::Error> {
    let template_var_paths = vec![
        ".template.json",
        "Cargo.toml",
        ".git/config",
    ];
    context::resolve_template_variables(&vec, &template_var_paths, HashMap::new())
}


fn main() -> Result<(), error::Error> {
    env_logger::init();
    let mut os_args = env::args_os().into_iter();
    // means we're running as cargo subcommand
    if env::var("CARGO").is_ok() {
        os_args.next();
    }
    let args = App::new(BIN_NAME)
        .version(VERSION)
        .about("Help!
        ")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(App::new("mit"))
        .subcommand(App::new("just"))
        .subcommand(App::new("just.lib.ts"))
        .subcommand(App::new("readme"))
        .subcommand(App::new("github-actions"))
        .arg(Arg::new("output")
            .short('o')
            .takes_value(true)
            .global(true)
            .about("Provide a file path, a directory with a trailing slash, or - for stdout.")
        )
        .arg(Arg::new("force")
            .long("force")
            .short('f')
            .global(true)
            .about("Overwrite existing files.")
        )
        .get_matches_from(os_args);

    let options = WritingOptions {
        force: args.is_present("force"),
    };


    let output_path = args.value_of("output").unwrap_or("./");

    let mut context = Context::new();
    let mut templates = Vec::new();
    let mut template_dir = Dir::new();

    match args.subcommand().unwrap() {
        ("mit", _) => {
            template_dir = templates.get_dir("mit").unwrap();
            context = resolve_template_variables(vec![
            ])?;
        }
        ("just", _) => {
            template_dir = templates.get_dir("just").unwrap();
            context = resolve_template_variables(vec![
            ])?;
        }
        ("just.lib.ts", _) => {
            template_dir = templates.get_dir("just.lib.ts").unwrap();
            context = resolve_template_variables(vec![
            ])?;
        }
        ("readme", _) => {
            template_dir = templates.get_dir("readme").unwrap();
            context = resolve_template_variables(vec![
                "github_repo",
                "name",
            ])?;
        }
        ("github-actions", _) => {
            template_dir = templates.get_dir("github-actions").unwrap();
            context = resolve_template_variables(vec![
            ])?;
        }
        ("clap", _) => {
            template_dir = templates.get_dir("clap").unwrap();
            context = resolve_template_variables(vec![
                "github_repo",
                "name",
            ])?;
        }
        _ => exit_with!("Template not recognized. Use --help for help."),
    }

    write_templates(template_dir, context, output_path, &options)
}
