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

refresh-export:
    cd ./backend && cargo test -p oceaniam-export
    rm -rf ./frontend/packages/sdk/src/types
    cp -r ./backend/crates/oceaniam-export/bindings/ ./frontend/packages/sdk/src/types

gen-sdks:
    cd ./backend && cargo run -p oceaniam -- openapi --output ../sdk/openapi.json
