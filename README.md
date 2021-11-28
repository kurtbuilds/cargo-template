# Cargo-template

Cargo-template makes it easy to add file templates to your project.

# Usage

    # Creates a LICENSE.md file with the MIT license.
    cargo template mit

    # You can specify a different output directory.
    cargo template mit -o a/different/directory/
    cargo template mit -o a/different/directory/DIFFERENT_FILE.md
    cargo template mit -o - > LICENSE.md

# Submissions

We welcome pull requests to add new file templates to this tool.