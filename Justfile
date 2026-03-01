default:
    just --list

gen-entities:
    sea-orm-cli generate entity -o ./backend/crates/oceaniam-database/src/model --with-serde both --enum-extra-derives ts_rs::TS --enum-extra-derives strum::Display --model-extra-derives ts_rs::TS

watch-backend:
    watchexec -e rs -r cargo run -p oceaniam

refresh-export:
    cd ./backend && cargo test -p oceaniam-export
    cp -r ./backend/crates/oceaniam-export/bindings ./frontend/src/utils/exports

