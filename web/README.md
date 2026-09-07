# OceanIAM Web

Vue 3 administration console styled with Tailwind CSS 4 and DaisyUI 5.

## Development

Requirements: Node.js 24 and Corepack.

```bash
corepack pnpm install
corepack pnpm dev
```

The Vite server listens on `http://127.0.0.1:8099` and proxies `/api` to the
backend at `http://127.0.0.1:8000`. Set `VITE_API_BASE_URL` to override the API
base URL.

## Checks

```bash
corepack pnpm typecheck
corepack pnpm lint
corepack pnpm test
corepack pnpm build
corepack pnpm test:e2e:mock
```

The mock smoke test runs on Firefox and is part of CI. The opt-in real-backend
smoke test requires an isolated migrated database and the one-time root password
printed by the migration command:

```bash
OCEANIAM_E2E_ROOT_PASSWORD='...' \
  corepack pnpm --filter @oceaniam/frontend exec playwright test \
  tests/e2e/real-smoke.spec.ts --project=firefox
```

The browser console uses JSON Bearer tokens persisted in local storage. Tokens
are proactively rotated near expiry, with single-flight retry and cross-tab
coordination.

## OpenAPI client

The private workspace package `@oceaniam/sdk` lives in `sdk/typescript/`.
Regenerate its checked-in contract and TypeScript types from the repository root:

```bash
just gen-openapi
```
