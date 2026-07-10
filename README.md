# OceanIAM

> [!NOTE]
> THIS PROJECT IS CURRENTLY UNDER DEVELOPMENT AND DOES NOT POSSESS OR PROVIDE
> PRODUCTION-GRADE CAPABILITIES.

ORBAC based IAM implemented in Rust.

## Design

You can find all related designs in [./docs/design](./docs/design)

## Frontend

All frontend source code is stored at [./frontend/](./frontend/) and powered by Flutter Web.

The web build resolves the backend URL in this order:

1. `OCEANIAM_BACKEND_URL` passed at build time via `--dart-define`.
2. Otherwise, at runtime, it falls back to `${window.location.origin}/api`,
   so the same built artifact can be deployed behind any reverse-proxy / gateway.

Use `fvm flutter build web --release` (or `just build-flutter`) to build it locally.

## Deploy

Environment files are split by runtime surface:

- Root compose stack: copy `.env.example` to `.env`.
- Local backend commands from `backend/`: copy `backend/.env.example` to `backend/.env`.
- Deployment compose from `deploy/`: copy `deploy/.env.example` to `deploy/.env`.

`backend/.env` uses `localhost` for PostgreSQL because cargo commands run on the
host. Root and deploy compose files use the `postgres` service hostname because
backend services run inside Docker.
