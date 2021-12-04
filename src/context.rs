/// functions to read context from the project environment (or user input)
use core::option::Option::Some;
use core::result::Result::{Err, Ok};
use ini::Ini;
use rustyline::Editor;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use toml::value::Value;
use tera::Context;
use crate::error::{Error};


/// insanely hackey, but its self contained, so its easy to extend when we need more data.
pub fn read_git_config(mut file: File) -> HashMap<String, String> {
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


/// insanely hackey, but its self contained, so its easy to extend when we need more data.
pub fn read_cargo_toml(mut file: File) -> HashMap<String, String> {
    let mut s: String = String::new();
    file.read_to_string(&mut s);
    let value = s.parse::<Value>().unwrap();
    let mut s = HashMap::new();
    s.insert("name".to_string(), value["package"]["name"].as_str().unwrap().to_string());
    s
}


pub fn read_context_file<'a>(
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
                eprintln!("{}: Resolved using {} to {}", k, path, &v);
                provided.insert(k, v);
            }
        });
    }
}


pub fn resolve_template_variables<'a>(
    var_names: &Vec<&'a str>,
    lookup_paths: &Vec<&'a str>,
    mut provided: HashMap<&'a str, String>,
) -> Result<Context, Error> {
    fill_empty_keys(var_names, lookup_paths, &mut provided);
    let mut editor = Editor::<()>::new();
    for name in var_names {
        if !provided.contains_key(name) {
            let readline = editor.readline(&format!("Provide value for {}: ", name))
                .map_err(|_| err!("Failed to get user input"))?;
            provided.insert(name, readline);
        }
    }
    let mut c = Context::new();
    for (k, v) in provided {
        c.insert(k, &v);
    }
    Ok(c)
}