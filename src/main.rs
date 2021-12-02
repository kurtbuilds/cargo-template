use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{MAIN_SEPARATOR, Path};
use clap::{App, AppSettings, Arg};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &'static str = env!("CARGO_BIN_NAME");


pub struct WritingOptions {
    pub force: bool,
}


struct FileTemplate<'a> {
    base_fpath: &'a Path,
    text: &'a str,
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
        File::create(&path)
            .expect(&format!("{}: Failed to create file.", path.display()))
            .write_all(file_template.text.as_bytes()).unwrap();
        eprintln!("{}: Wrote file.", path.display());
    };
}


fn main() {
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
            let template = FileTemplate {
                base_fpath: Path::new("LICENSE.md"),
                text: include_str!("../template/mit/LICENSE.md"),
            };
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        ("just", _) => {
            let template = FileTemplate {
                base_fpath: Path::new("Justfile"),
                text: include_str!("../template/just/Justfile"),
            };
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        ("just.lib.ts", _) => {
            let template = FileTemplate {
                base_fpath: Path::new("Justfile"),
                text: include_str!("../template/just.lib.ts/Justfile"),
            };
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        ("readme", _) => {
            let template = file_template!("../template/readme/README.md");
            //     base_fpath: Path::new("README.md"),
            //     text: include_str!(),
            // };
            let output = args.value_of("output")
                .map(|s| Path::new(s))
                .unwrap_or(template.base_fpath);
            write_single(&template, output, &options)
        }
        _ => exit_with!("Template not recognized. Use --help for help."),
    }
}
