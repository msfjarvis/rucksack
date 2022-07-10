alias b := build
alias r := run
alias v := version

name := `dasel select -f Cargo.toml -s package.name`
version := `dasel select -f Cargo.toml -s package.version`

set positional-arguments := true
set dotenv-load := true

build type="":
    cargo build {{ type }}

run type="":
    cargo run {{ type }}

version:
    @echo {{ version }}
