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
    rm -rf ./sdk/typescript/src/types
    cp -r ./backend/crates/oceaniam-export/bindings/ ./sdk/typescript/src/types
    cd ./sdk/typescript && pnpm build

fmt:
    cd ./backend && cargo fmt
    cd ./frontend && pnpm fmt
    cd ./sdk/rust && cargo fmt
    cd ./sdk/typescript && pnpm fmt
    cd ./sdk/dart && fvm dart format .

build:
    cd ./backend && cargo build --all -r
    cd ./frontend && pnpm build
    cd ./sdk/dart && fvm dart run build_runner build

build-flutter:
    cd ./sdk/dart && fvm dart run build_runner build

check:
    cd ./backend && cargo test --all -r
    cd ./backend && cargo build --all -r
    cd ./frontend && pnpm build
    cd ./sdk/rust && cargo test --all -r
    cd ./sdk/rust && cargo build --all -r
    cd ./sdk/typescript && pnpm build
