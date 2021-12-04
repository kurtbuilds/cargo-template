<div id="top"></div>

<p align="center">
<a href="https://github.com/kurtbuilds/cargo-template/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kurtbuilds/cargo-template.svg?style=flat-square" alt="GitHub Contributors" />
</a>
<a href="https://github.com/kurtbuilds/cargo-template/stargazers">
    <img src="https://img.shields.io/github/stars/kurtbuilds/cargo-template.svg?style=flat-square" alt="Stars" />
</a>
<a href="https://github.com/kurtbuilds/cargo-template/actions">
    <img src="https://img.shields.io/github/workflow/status/kurtbuilds/cargo-template/test?style=flat-square" alt="Build Status" />
</a>
<a href="https://crates.io/crates/cargo-template">
    <img src="https://img.shields.io/crates/d/cargo-template?style=flat-square" alt="Downloads" />
</a>
<a href="https://crates.io/crates/cargo-template">
    <img src="https://img.shields.io/crates/v/cargo-template?style=flat-square" alt="Crates.io" />
</a>

</p>

# Cargo-Template

`cargo-template` makes it easy to add file templates to your project.

# Usage

Creates a LICENSE.md file with the MIT license.

    cargo template mit

You can specify a different output directory.

    cargo template mit -o a/different/directory/
    cargo template mit -o a/different/directory/DIFFERENT_FILE.md
    cargo template mit -o - > LICENSE.md  # stdout

You can see a list of all offered templates.

    cargo template --help

We always welcome more! Please submit PRs to share great file templates with other Rustaceans (and others!)

# Installation

    cargo install cargo-template


# Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
