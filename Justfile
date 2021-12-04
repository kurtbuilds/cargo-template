set dotenv-load := false

help:
    @just --list --unsorted

build:
    checkexec target/debug/cargo-template $(fd '.*' templates) -- cargo clean -p cargo-template
    cargo build
alias b := build

run *args:
    checkexec target/debug/cargo-template $(fd '.*' templates) -- cargo clean -p cargo-template
    cargo run {{args}}
alias r := run

release:
    checkexec target/release/cargo-template $(fd '.*' templates) -- cargo clean --release -p cargo-template
    cargo build --release

install:
    checkexec ~/.cargo/bin/cargo-template $(fd '.*' templates) -- cargo clean --release -p cargo-template
    cargo install --path .

bootstrap:
    cargo install cargo-edit

test *args:
    cargo test {{args}}

check:
    cargo check
alias c := check

fix:
    cargo clippy --fix

# Bump version. level=major,minor,patch
version level:
    git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.
    cargo set-version --bump {{level}}
    cargo update # This bumps Cargo.lock
    VERSION=$(rg  "version = \"([0-9.]+)\"" -or '$1' Cargo.toml | head -n1) && \
        git commit -am "Bump {{level}} version to $VERSION" && \
        git tag v$VERSION && \
        git push origin v$VERSION
    git push

publish:
    cargo publish

patch: test
    just version patch
    just publish
