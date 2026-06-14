# OceanIAM

> [!NOTE]
> THIS PROJECT IS CURRENTLY UNDER DEVELOPMENT AND DOES NOT POSSESS OR PROVIDE
> PRODUCTION-GRADE CAPABILITIES.

ORBAC based IAM implemented in Rust.

## Design

You can find all related designs in [./docs/design](./docs/design)

## Frontend

All frontend source code storaged at [./frontend/](./frontend/) and powered by Vue3 and daisyUI.

## Deploy

Environment files are split by runtime surface:

- Root compose stack: copy `.env.example` to `.env`.
- Local backend commands from `backend/`: copy `backend/.env.example` to `backend/.env`.
- Deployment compose from `deploy/`: copy `deploy/.env.example` to `deploy/.env`.

`backend/.env` uses `localhost` for PostgreSQL because cargo commands run on the
host. Root and deploy compose files use the `postgres` service hostname because
backend services run inside Docker.
