use std::path::Path;

fn main() {
    println!("{}, {}",
             Path::new("a/b/c").is_dir(),
             Path::new("a/b/c/").is_dir(),
    );
}