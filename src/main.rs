#[macro_use]
pub mod error;
pub mod context;

use std::{env, fs};
use std::collections::HashMap;
use std::fs::Permissions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::PermissionsExt;
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
    pub verbose: bool,
}


struct CompletedFile {
    final_path: PathBuf,
    rendered: String,
    permissions: u32,
}


fn write_templates(tera: &mut Tera, template_group: &str, context: Context, output_path: &str, options: &WritingOptions) -> Result<()> {
    let template_names = {
        tera.get_template_names()
            .filter(|name| name.starts_with(&(template_group.to_string() + "/")))
            .map(|s| s.to_string()).collect::<Vec<_>>()
    };

    eprintln!("Found {} templates in group {}:\n{}", template_names.len(), template_group, template_names.iter()
        .map(|s| &s[template_group.len() + 1..])
        .collect::<Vec<_>>()
        .join("\n")
    );

    let to_dir = output_path.ends_with(&MAIN_SEPARATOR.to_string());
    let to_stdout = output_path == "-";
    if template_names.len() > 1 && (to_stdout || !to_dir) {
        return Err(err!("Error: You are trying to write multiple file templates, but the output is not a directory. Got: {}", output_path).into());
    }
    let output_path = Path::new(output_path);

    let mut final_files = Vec::new();
    for name in template_names {
        let raw_path = if to_dir {
            let relative_path = Path::new(&name).components().skip(1).collect::<PathBuf>();
            output_path.join(relative_path)
        } else {
            output_path.to_owned()
        };

        final_files.push(CompletedFile {
            final_path: PathBuf::from(tera.render_str(raw_path.to_str().unwrap(), &context)?),
            rendered: tera.render(&name, &context)?,
            permissions: if name == "github-actions/.github/package" { 0x755 } else { 0o644 },
        });
        if options.verbose {
            eprintln!("Rendering template {} to {}", name, raw_path.display());
        }
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
        let mut f = fs::File::create(&file.final_path)
            .map_err(|_| err!("{}: Failed to create file.", file.final_path.display()))?;
        f.write_all(file.rendered.as_bytes())
            .map_err(|_| err!("{}: Failed to write to file.", file.final_path.display()))?;
        f.set_permissions(Permissions::from_mode(file.permissions))?;
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
    if verbose {
        eprintln!("");
    }
    Ok(tera)
}


fn find_first_root() -> Option<PathBuf> {
    for fpath in vec![
        "mod.rs",
        "src/main.rs",
        "src/lib.rs",
    ] {
        if fs::metadata(fpath).is_ok() {
            return Some(PathBuf::from(fpath));
        }
    }
    return None;
}


fn add_mod_to_file(mut s: String, name: &str, public: bool) -> String {
    s.insert_str(0, &format!(
        "{}mod {};\n",
        if public { "pub " } else { "" },
        name,
    ));
    s
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
        .subcommand(App::new("mod")
            .arg(Arg::new("pub")
                .long("pub")
                .short('p')
                .required(false))
            .arg(Arg::new("dir")
                .long("dir")
                .short('d')
                .takes_value(false)
                .required(false))
        )
        .arg(Arg::new("output")
            .short('o')
            .takes_value(true)
            .global(true)
            .help("Provide a file path, a directory with a trailing slash, or - for stdout.")
        )
        .arg(Arg::new("force")
            .long("force")
            .short('f')
            .global(true)
            .help("Overwrite existing files.")
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
        verbose: args.is_present("verbose"),
    };

    let output_path = args.value_of("output").unwrap_or("./");
    let template_group = args.subcommand().unwrap().0;
    let mut tera = register_templates(verbose)?;

    // some templates have special casing right now.
    match args.subcommand().unwrap() {
        ("mod", matches) => {
            let context = resolve_template_variables(vec!["mod_name"], verbose)?;
            let first_root = find_first_root().ok_or(anyhow::anyhow!("Could not find a mod root at the current path"))?;
            let mut f = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .append(false)
                .open(&first_root)
                .unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();

            let mod_name = context.get("mod_name").unwrap().as_str().unwrap().to_owned();
            // add pub mod <name>; statement to the parent.
            let s = add_mod_to_file(s, &mod_name, matches.is_present("pub"));
            f.seek(SeekFrom::Start(0))?;
            f.write_all(s.as_bytes())?;
            drop(f);

            // create the <name>.rs or the <name>/mod.rs file.
            let mut output_path = PathBuf::from(output_path);
            if vec!["src/main.rs", "src/lib.rs"].contains(&first_root.to_str().unwrap()) {
                output_path.push("src/");
            }
            if matches.is_present("dir") {
                output_path.push(mod_name);
                output_path.push("mod.rs");
            }
            write_templates(&mut tera, template_group, context, output_path.to_str().unwrap(), &options)?;
            return Ok(());
        }
        _ => {}
    }

    // generic handler
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
