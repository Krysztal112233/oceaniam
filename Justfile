default:
    just --list

gen-entities:
    test "$(sea-orm-cli --version)" = "sea-orm-cli 2.0.1" || (echo "sea-orm-cli 2.0.1 is required; install it with: cargo install sea-orm-cli --version 2.0.1 --locked" >&2; exit 1)
    cd ./backend && \
    sea-orm-cli generate entity -o ./crates/oceaniam-database/src/model \
    --with-serde both \
    --entity-format compact \
    --enum-extra-derives Hash \
    --enum-extra-derives strum::Display

watch-backend:
    cd ./backend && watchexec -e rs -r cargo run -p oceaniam

fmt:
    cd ./backend && cargo fmt
    cd ./sdk/rust && cargo fmt
    cd ./frontend && fvm dart format .
    cd ./sdk/dart && fvm dart format .

build:
    cd ./backend && cargo build --all -r
    cd ./sdk/dart && fvm dart run build_runner build
    cd ./frontend && fvm dart run build_runner build

build-flutter:
    cd ./sdk/dart && fvm dart run build_runner build
    cd ./frontend/ && fvm dart run build_runner build
    cd ./frontend/ && fvm flutter build web --release

check:
    cd ./backend && cargo test --all -r
    cd ./backend && cargo build --all -r
    cd ./sdk/rust && cargo test --all -r
    cd ./sdk/rust && cargo build --all -r
    cd ./sdk/dart && fvm dart test
    cd ./frontend && fvm dart analyze
    cd ./frontend && fvm flutter build web --release
