# AGENTS.md

## Project Facts

- This workspace is a Vue 3 + Vite + TypeScript frontend.
- UI is built with Tailwind CSS v4 + DaisyUI.
- State management uses Pinia with `pinia-plugin-persistedstate`.
- API access should go through the local workspace SDK in `sdk/typescript` whenever possible.
- The current product model is centered around `tenant` and `application` context.

## Directory Expectations

- `src/view`: route-level pages and page-owned data loading.
- `src/components`: reusable presentational or interaction components.
- `src/stores`: shared cross-page state only.
- `sdk/typescript`: generated types and API client used by the frontend.

## State Rules

- Keep authentication state in `src/stores/auth.ts`.
- Keep shared tenant context in `src/stores/tenant.ts`.
- Keep page-local state in the page unless it is truly shared across routes or navbar-level UI.
- Do not move every API response into Pinia by default.
- Lists, filters, dialogs, loading flags, and form state should stay local unless multiple pages need the same source of truth.

## Tenant Context Rules

- `currentTenantId` belongs to the tenant store.
- Components must not maintain separate competing versions of current tenant state.
- If a page depends on tenant scope, it should read tenant context from the tenant store and stay consistent with routing.
- The tenant switcher should reuse tenant store state instead of loading its own duplicate tenant state.
- If persisted tenant state becomes invalid, fall back to a valid tenant from the fetched tenant list.

## Routing Rules

- `/applications` is an entry route that restores the last usable tenant context.
- `/tenants/:tenantId/applications` is the tenant-scoped applications view.
- New tenant-scoped pages should follow the same pattern unless there is a strong reason not to.
- Do not leave dead navigation targets in the sidebar or navbar.

## SDK Rules

- Prefer `@oceaniam/sdk` client methods over ad hoc `fetch` calls.
- If a type is missing from the SDK public entrypoint, temporarily import the generated type directly only when necessary, then consider whether the SDK export should be fixed.
- Be careful with generated request types: some pagination fields are `bigint`, not `number`.
- Do not silently change SDK-generated types in frontend code to make usage easier.

## Dependency Rules

- Ask for user approval before adding any new dependency.
- Do not silently install packages, switch icon libraries, or introduce helper libraries just because they are convenient.
- If a new dependency seems necessary, explain why the current repo cannot reasonably solve the problem without it.

## Component Rules

- Route views own data fetching and page-level state transitions.
- Table and list components should stay presentational.
- Row-level action groups may be extracted into dedicated components.
- Do not introduce a generic DataTable abstraction unless the column model and interaction patterns are already stable across multiple screens.
- Extract shared layout shells only when multiple pages already share the same structure.

## UI Rules

- Reuse the existing Tailwind + DaisyUI approach.
- When introducing an icon, prefer an Iconify component first.
- Do not add a new icon library if Iconify already covers the need.
- If Iconify does not provide a suitable icon, prepare a small set of replacement candidates and ask the user which direction they want before proceeding.
- Do not add inline SVGs as the default approach when an Iconify component is available.
- For wide tables, prefer horizontal scrolling before inventing a separate mobile card view.
- Preserve the current visual language unless the task explicitly asks for a redesign.

## Tenant Switching UX Rules

- Tenant switching should feel like persistent context, not a one-off page control.
- The app should remember the last valid tenant when that improves navigation flow.
- Switching tenant should update the application view consistently and avoid duplicate source-of-truth logic.
- If there are no tenants, show explicit empty-state messaging instead of failing silently.

## Change Discipline

- Keep each change as atomic as practical.
- Do not bundle unrelated refactors, cleanup, or style churn into the same change unless the user explicitly asks for it.
- If the working tree already contains large or noisy changes, warn the user before proceeding.

## Verification

- After meaningful frontend changes, run `pnpm build`.
- If a route, store contract, or SDK usage changes, verify the affected page still compiles and the navigation path still makes sense.
- If a workaround is required because of current repo limitations, state it explicitly in the final summary.
- If large pre-existing worktree changes are relevant to the task, mention them in the final summary.
