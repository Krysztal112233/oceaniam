# OceanIAM

> [!NOTE]
> THIS PROJECT IS CURRENTLY UNDER DEVELOPMENT AND DOES NOT POSSESS OR PROVIDE
> PRODUCTION-GRADE CAPABILITIES.

ORBAC based IAM implemented in Rust.

## Design

You can find all related designs in [./docs/design](./docs/design)

## Frontend

The active administration console is stored at [./web/](./web/) and powered by
Vue 3, TypeScript, Tailwind CSS, and DaisyUI. The previous Flutter client remains
in [./frontend/](./frontend/) while the migration is evaluated.

The web application uses `/api` by default, matching the bundled Nginx gateway.
For a different deployment, set `VITE_API_BASE_URL` at build time.

```bash
corepack pnpm install
corepack pnpm dev
```

Use `just build-web` for a production build. The TypeScript SDK in
`sdk/typescript/` is generated from the backend OpenAPI document; run
`just gen-openapi` after changing endpoint contracts.

## Deploy

Environment files are split by runtime surface:

- Root compose stack: copy `.env.example` to `.env`.
- Local backend commands from `backend/`: copy `backend/.env.example` to `backend/.env`.
- Deployment compose from `deploy/`: copy `deploy/.env.example` to `deploy/.env`.

`backend/.env` uses `localhost` for PostgreSQL because cargo commands run on the
host. Root and deploy compose files use the `postgres` service hostname because
backend services run inside Docker.

Both compose stacks require `OCEANIAM_PUBLIC_BASE_URL` before startup. The root
example uses the local nginx gateway at `http://localhost:8900`; deployments must
replace the empty value in `deploy/.env.example` with their canonical HTTPS origin.
