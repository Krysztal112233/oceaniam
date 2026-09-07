# OceanIAM Web

- This directory is the Vue 3 + strict TypeScript administration console.
- Use Node.js 24 and run package commands through the root pnpm workspace.
- Prefer DaisyUI components and Tailwind utilities; do not introduce a parallel
  component system or custom theme without an explicit design decision.
- Pinia owns client state only. Remote data belongs in TanStack Vue Query, and
  tenant-scoped query keys start with `tenant` followed by the tenant ID.
- API calls must use the generated `@oceaniam/sdk` contract. Do not duplicate
  backend request or response interfaces by hand.
- Keep the backend base path relative (`/api`) unless an explicit
  `VITE_API_BASE_URL` is provided.
- Preserve the single-expanded-card behavior for application and secret lists.
- New visible text must be added to both `zh-CN` and `en-US` messages.
- Before handoff, run typecheck, lint, unit tests, production build, and relevant
  browser flows. Use Firefox DevTools MCP for rendered UI verification.
