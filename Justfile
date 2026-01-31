default:
    just --list

gen-entities:
    sea-orm-cli generate entity -o ./crates/oceaniam-database/src/model --with-serde both --enum-extra-derives ts_rs::TS --enum-extra-derives strum::Display --model-extra-derives ts_rs::TS

watch-backend:
    watchexec -e rs -r cargo run -p oceaniam

