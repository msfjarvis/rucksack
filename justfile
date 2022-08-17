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

log:
    journalctl --user -xeu file-collector.service

run type="":
    cargo run {{ type }}

start:
    systemctl --user start file-collector.service

status:
    systemctl --user status file-collector.service

stop:
    systemctl --user stop file-collector.service

test:
    cargo nextest run

version:
    @echo {{ version }}
