alias b := build
alias c := check
alias r := run
alias v := version

name := `dasel select -f Cargo.toml -s package.name`
version := `dasel select -f Cargo.toml -s package.version`

set positional-arguments := true
set dotenv-load := true

build type="":
    cargo build {{ type }}

check type="":
    cargo check {{ type }}

run type="":
    cargo run {{ type }}

test:
    cargo nextest run

version:
    @echo {{ version }}
