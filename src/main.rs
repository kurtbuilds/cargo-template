use std::{env, fs};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::{MAIN_SEPARATOR, Path};
use std::process::exit;
use clap::{App, AppSettings, Arg};
use askama::Template;
use serde::{Deserialize, Serialize};
use rustyline::Editor;
use anyhow::Result;
use ini::Ini;
use log::info;
use toml::Value;


const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &'static str = env!("CARGO_BIN_NAME");


pub struct WritingOptions {
    pub force: bool,
}


struct FileTemplate<'a> {
    base_fpath: &'a Path,
    pub text: &'a str,
}

macro_rules! file_template {
    ($base_fpath:expr) => {
        FileTemplate {
            base_fpath: Path::new($base_fpath).file_name().unwrap().as_ref(),
            text: include_str!($base_fpath),
        }
    };
}

macro_rules! exit_with {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        std::process::exit(1);
    })
}


#[derive(Template, Deserialize)] // this will generate the code...
#[template(path = "readme/README.md")]
struct ReadmeTemplate<'a> {
    github_repo: &'a str,
    name: &'a str,
}


fn write_multiple<'a>(files: &'a Vec<FileTemplate>, output_path: &Path, options: &WritingOptions) {
    if !output_path.ends_with(MAIN_SEPARATOR.to_string()) {
        exit_with!("Output path must end with a path separator.");
    }
    files.iter().map(|file| {
        let path = output_path.join(file.base_fpath);
        if path.is_file() && !options.force {
            exit_with!("{}: File already exists.", path.display());
        }
        (&file.text, path)
    }).for_each(|(text, path)| {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        File::create(path.as_path())
            .expect(&format!("{}: Failed to create file.", path.display()))
            .write_all(text.as_bytes())
            .expect(&format!("{}: Failed to write to file.", path.display()));
        eprintln!("{}: Wrote file.", path.display());
    });
}


fn write_single(file_template: &FileTemplate, output_path: &Path, options: &WritingOptions) {
    if output_path.to_str().unwrap() == "-" {
        std::io::stdout()
            .write_all(file_template.text.as_bytes()).unwrap();
    } else {
        let path = if output_path.ends_with(MAIN_SEPARATOR.to_string()) {
            Path::new(output_path).join(file_template.base_fpath)
        } else {
            Path::new(output_path).to_path_buf()
        };
        if path.is_file() && !options.force {
            exit_with!("{}: File already exists.", path.display());
        }
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        File::create(&path)
            .expect(&format!("{}: Failed to create file.", path.display()))
            .write_all(file_template.text.as_bytes()).unwrap();
        eprintln!("{}: Wrote file.", path.display());
    };
}



fn read_git_config(mut file: File) -> HashMap<String, String> {
    let mut s: String = String::new();
    file.read_to_string(&mut s);
    let conf = Ini::load_from_str(&s).unwrap();
    let mut s = HashMap::new();
    if let Some(section) = conf.section(Some("remote \"origin\"".to_string())) {
        if let Some(url) = section.get("url") {
            s.insert("repo".to_string(), url.to_string());
            if url.contains("github.com") {
                s.insert("github_repo".to_string(), url.split("github.com/").skip(1).next().unwrap()
                    .split(".git").next().unwrap()
                    .to_string());
            }
        }
    }
    s
}


fn read_cargo_toml(mut file: File) -> HashMap<String, String> {
    let mut s: String = String::new();
    file.read_to_string(&mut s);
    let value = s.parse::<Value>().unwrap();
    let mut s = HashMap::new();
    s.insert("name".to_string(), value["package"]["name"].as_str().unwrap().to_string());
    s
}


fn read_context_file<'a>(
    var_names: &Vec<&'a str>,
    path: &str,
) -> HashMap<&'a str, String> {
    match File::open(path) {
        Ok(file) => {
            let map: HashMap<String, String> = if path.ends_with(".git/config") {
                read_git_config(file)
            } else if path.ends_with("Cargo.toml") {
                read_cargo_toml(file)
            } else {
                serde_json::from_reader(file).unwrap()
            };
            var_names.iter()
                .filter_map(|k| map.get(&k.to_string()).map(|v| (*k, v.to_string())))
                .collect()
        }
        Err(_) => HashMap::new(),
    }
}


fn fill_empty_keys<'a>(
    var_names: &Vec<&'a str>,
    lookup_paths: &Vec<&'a str>,
    provided: &mut HashMap<&'a str, String>,
) {
    for path in lookup_paths {
        let context = read_context_file(&var_names, path);
        context.into_iter().for_each(|(k, v)| {
            if !provided.contains_key(k) {
                info!("{}: Resolved using {} to {}", k, path, &v);
                provided.insert(k, v);
            }
        });
    }
}


fn resolve_template_variables<'a>(
    var_names: &Vec<&'a str>,
    lookup_paths: &Vec<&'a str>,
    mut provided: HashMap<&'a str, String>,
) -> Result<HashMap<&'a str, String>> {
    fill_empty_keys(var_names, lookup_paths, &mut provided);
    let mut editor = Editor::<()>::new();
    for name in var_names {
        if !provided.contains_key(name) {
            let readline = editor.readline(&format!("Provide value for {}: ", name))?;
            provided.insert(name, readline);
        }
    }
    Ok(provided)
}


fn main() {
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
    match args.subcommand().unwrap() {
        ("mit", _) => {
            let template = file_template!("../templates/mit/LICENSE.md");
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        ("just", _) => {
            let template = FileTemplate {
                base_fpath: Path::new("Justfile"),
                text: include_str!("../templates/just/Justfile"),
            };
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        ("just.lib.ts", _) => {
            let template = FileTemplate {
                base_fpath: Path::new("Justfile"),
                text: include_str!("../templates/just.lib.ts/Justfile"),
            };
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        ("readme", _) => {
            let mut template = file_template!("../templates/readme/README.md");
            let vars = resolve_template_variables(
                &vec![
                    "github_repo",
                    "name",
                ],
                &vec![
                    ".template.json",
                    "Cargo.toml",
                    ".git/config",
                ],
                HashMap::new(),
            ).map_err(|e| {
                exit(1);
            }).unwrap();
            let s = serde_json::to_string(&vars).unwrap();
            let readme: ReadmeTemplate = serde_json::from_str(&s).unwrap();
            let text = readme.render().unwrap();
            template.text = &text;
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        ("github-actions", _) => {
            let template = FileTemplate {
                base_fpath: Path::new(".github/workflows/test.yaml"),
                text: include_str!("../templates/github-actions/.github/workflows/test.yaml"),
            };
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        _ => exit_with!("Template not recognized. Use --help for help."),
    }
}
