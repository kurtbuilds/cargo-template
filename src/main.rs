#[macro_use]
pub mod error;
pub mod context;

use std::{env, fs};
use std::collections::HashMap;
use std::io::{Write};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use clap::{App, AppSettings, Arg};

use tera::{Context, Tera};
use include_dir::{include_dir, Dir, DirEntry};
use anyhow::Result;


static VERSION: &str = env!("CARGO_PKG_VERSION");
static BIN_NAME: &str = env!("CARGO_BIN_NAME");
static TEMPLATE_DIR: Dir<'_> = include_dir!("templates");


pub struct WritingOptions {
    pub force: bool,
}


struct CompletedFile {
    final_path: PathBuf,
    rendered: String,
}


fn write_templates(tera: &mut Tera, template_group: &str, context: Context, output_path: &str, options: &WritingOptions) -> Result<()> {
    let template_names = {
        tera.get_template_names().filter(|name| name.starts_with(&(template_group.to_string() + "/"))).map(|s| s.to_string()).collect::<Vec<_>>()
    };
    eprintln!("Found {} templates in group {}: {}", template_names.len(), template_group, template_names.join(", "));

    let to_dir = output_path.ends_with(&MAIN_SEPARATOR.to_string());
    let to_stdout = output_path == "-";
    if template_names.len() > 1 && (to_stdout || !to_dir) {
        return Err(err!("Error: You are trying to write multiple file templates, but the output is not a directory. Got: {}", output_path).into());
    }
    let output_path = Path::new(output_path);

    let mut final_files = Vec::new();
    for name in template_names {
        let relative_path = Path::new(&name).components().skip(1).collect::<PathBuf>();
        final_files.push(CompletedFile {
            final_path: if to_dir { output_path.join(relative_path).to_owned() } else { output_path.to_owned() },
            rendered: tera.render(&name, &context)?,
        });
    }

    if to_stdout {
        for file in &final_files {
            std::io::stdout()
                .write_all(file.rendered.as_bytes()).unwrap();
            return Ok(());
        }
    }

    for file in &final_files {
        if file.final_path.is_file() && !options.force {
            return Err(err!("{}: File already exists.", file.final_path.display()).into());
        }
    }

    for file in &final_files {
        fs::create_dir_all(file.final_path.parent().unwrap()).unwrap();
        fs::File::create(&file.final_path)
            .map_err(|_| err!("{}: Failed to create file.", file.final_path.display()))?
            .write_all(file.rendered.as_bytes())
            .map_err(|_| err!("{}: Failed to write to file.", file.final_path.display()))?;
        eprintln!("{}: Wrote file.", file.final_path.display());
    }
    Ok(())
}


fn resolve_template_variables(vec: Vec<&str>, verbose: bool) -> Result<Context, error::Error> {
    let template_var_paths = vec![
        ".template.json",
        "Cargo.toml",
        ".git/config",
    ];
    context::resolve_template_variables(&vec, &template_var_paths, HashMap::new(), verbose)
}


fn register_templates_recurse(dir: &Dir) -> Vec<(String, String)> {
    let mut templates = Vec::new();
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(d) => {
                let mut subdir_templates = register_templates_recurse(d);
                templates.append(&mut subdir_templates);
            }
            DirEntry::File(f) => {
                templates.push((
                    f.path().to_str().unwrap().to_owned(), f.contents_utf8().unwrap().to_string(),
                ));
            }
        }
    }
    templates
}


fn register_templates(verbose: bool) -> Result<Tera> {
    let mut tera = Tera::default();
    tera.add_raw_templates(register_templates_recurse(&TEMPLATE_DIR).into_iter()
        .map(|(name, body)| {
            if verbose {
                eprintln!("{}: Found template", &name)
            }
            (name, body)
        }))?;
    Ok(tera)
}


fn main() -> Result<()> {
    let mut os_args = env::args_os();
    // means we're running as cargo subcommand
    if let Ok(last_run_executable) = env::var("_") {
        if let Some(user_provided_bin) = env::args().next() {
            if user_provided_bin.starts_with(&last_run_executable) {
                os_args.next();
            }
        }
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
        .arg(Arg::new("verbose")
            .long("verbose")
            .short('v')
            .global(true)
        )
        .get_matches_from(os_args);

    let verbose = args.is_present("verbose");
    let options = WritingOptions {
        force: args.is_present("force"),
    };

    let output_path = args.value_of("output").unwrap_or("./");
    let template_group = args.subcommand().unwrap().0;
    let mut tera = register_templates(verbose)?;

    let required_vars = match args.subcommand().unwrap() {
        ("mit", _) => {
            vec![]
        }
        ("just", _) => {
            vec![]
        }
        ("just.lib.ts", _) => {
            vec![]
        }
        ("readme", _) => {
            vec![
                "github_repo",
                "name",
            ]
        }
        ("github-actions", _) => {
            vec![]
        }
        ("clap", _) => {
            vec![
                "github_repo",
                "name",
            ]
        }
        _ => {
            eprintln!("Template not recognized. Use --help for help.");
            std::process::exit(1)
        }
    };

    let context = resolve_template_variables(required_vars, verbose)?;
    write_templates(&mut tera, template_group, context, output_path, &options)
}
