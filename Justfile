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
    cd ./sdk/dart && fvm dart format .
    corepack pnpm format

build:
    cd ./backend && cargo build --all -r
    cd ./sdk/rust && cargo build --all -r
    corepack pnpm build

build-web:
    corepack pnpm build

gen-openapi:
    cd ./backend && cargo run -p oceaniam --release -- openapi --output ../sdk/typescript/openapi.json
    corepack pnpm generate:api

check-openapi: gen-openapi
    git diff --exit-code -- sdk/typescript/openapi.json sdk/typescript/src/schema.ts

check:
    cd ./backend && cargo test --all -r
    cd ./backend && cargo build --all -r
    cd ./sdk/rust && cargo test --all -r
    cd ./sdk/rust && cargo build --all -r
    cd ./sdk/dart && fvm dart test
    corepack pnpm check
