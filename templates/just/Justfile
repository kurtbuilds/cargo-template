set dotenv-load := false

help:
    @just --list --unsorted

bootstrap:
    cargo install cargo-edit

check:
    cargo check

build:
    cargo build
alias b := build

test *args:
    cargo test {{args}}

run *args:
    cargo run {{args}}
alias r := run

release:
    cargo build --release

fix:
    cargo clippy --fix

install:
    cargo install --path .

# Bump version. level=major,minor,patch
version level:
    git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.
    cargo set-version --bump {{level}}
    cargo update # This bumps Cargo.lock
    VERSION=$(rg  "version = \"([0-9.]+)\"" -or '$1' Cargo.toml | head -n1) && \
        git commit -am "Bump version to $VERSION" && \
        git tag v$VERSION && \
        git push origin v$VERSION
    git push

publish:
    cargo publish

patch: test
    just version patch
    just publish