default:
    just --list

gen-entities:
    cd ./backend && \
    sea-orm-cli generate entity -o ./crates/oceaniam-database/src/model \
    --with-serde both \
    --enum-extra-derives Hash \
    --enum-extra-derives strum::Display \
    --enum-extra-derives strum::EnumString

watch-backend:
    cd ./backend && watchexec -e rs -r cargo run -p oceaniam

export:
    cd ./backend && cargo test -p oceaniam-export
    rm -rf ./sdk/dart/lib/src/types
    cp -r ./backend/crates/oceaniam-export/bindings/ ./sdk/dart/lib/src/types
    cd ./sdk/dart && fvm dart run build_runner build

fmt:
    cd ./backend && cargo fmt
    cd ./sdk/rust && cargo fmt
    cd ./flutter_frontend && fvm dart format .
    cd ./sdk/dart && fvm dart format .

build:
    cd ./backend && cargo build --all -r
    cd ./sdk/dart && fvm dart run build_runner build
    cd ./flutter_frontend && fvm dart run build_runner build

build-flutter:
    cd ./sdk/dart && fvm dart run build_runner build
    cd ./flutter_frontend/ && fvm dart run build_runner build
    cd ./flutter_frontend/ && fvm flutter build web --release

check:
    cd ./backend && cargo test --all -r
    cd ./backend && cargo build --all -r
    cd ./sdk/rust && cargo test --all -r
    cd ./sdk/rust && cargo build --all -r
    cd ./sdk/dart && fvm dart test
    cd ./flutter_frontend && fvm dart analyze
    cd ./flutter_frontend && fvm flutter build web --release
