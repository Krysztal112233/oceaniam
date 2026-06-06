# Backend Agent Instructions

These instructions apply to work performed inside the `backend` project.

## Authorization Model

- This backend implements an ORBAC-based IAM model.
- When reasoning about authorization behavior, prefer ORBAC terminology instead of reducing the system to a simple RBAC-style user-role-permission mapping.
- `organization`: the organizational boundary where an authorization rule applies.
- `role`: the responsibility or function a subject holds within an organization.
- `activity`: the business action governed by an authorization rule.
- `view`: the resource view or resource set targeted by an authorization rule.
- `context`: the condition or situation under which an authorization rule becomes effective.
- If a change introduces or modifies authorization semantics, make sure the corresponding `organization`, `role`, `activity`, `view`, and `context` meaning is clear before implementation.

## Change Scope

- Keep every change as small as possible.
- Limit edits to the minimum required to complete the current task.
- Do not mix unrelated refactors, opportunistic cleanup, or cross-module changes into the same task.

## Existing Backend Changes

- If the workspace already contains a significant amount of pending backend-related changes, warn the user before continuing.
- Ask the user to save or explicitly confirm those existing changes first.
- Do not proceed with further backend work until the user has explicitly agreed.

## Unit Tests

- Any unit test written by the agent must include `// NOTE: AI-generated test` immediately above the test.
- When a file already contains tests, read them. Some tests exist to prove equivalence between two
  approaches and mark which one is preferred (`// NOTE: PREFER THIS`). Use that preferred pattern
  rather than the alternative being compared.

## Design Responsibility

- Focus on helping the user complete tedious, repetitive, or execution-heavy work.
- Do not replace the user in system design, architecture design, or major solution-shaping decisions.
- If the user is asking for system design work, instruct them to enter Plan Mode first, complete the plan, and only then move on to implementation.

## Backend and Frontend Isolation

- When making backend changes, do not modify files under `frontend/`.
- Treat `backend` and `frontend` as independently developed projects unless the user explicitly requests coordinated cross-project work.

## Endpoint Documentation Consistency

- When helping implement or modify endpoints in `crates/oceaniam/src/endpoints/`, update the corresponding `utoipa` annotations in the same change.
- If an endpoint's path, method, parameters, request body, response body, tags, or security behavior changes, update the related `#[utoipa::path(...)]` attributes so the OpenAPI description stays accurate.
- If a newly added or modified endpoint is exposed through the frontend SDK, update `sdk/typescript/src/client.ts` in the same change so the SDK client stays in sync with the backend API surface.

## VO Export Sync

- Any change to a value-object type in `crates/oceaniam-vo/src/` that carries `#[derive(ts_rs::TS)]` must have a corresponding export stub added or updated in `crates/oceaniam-export/src/vo/`.
- The stub file mirrors the VO crate's module structure one-to-one.  A missing stub means the TypeScript type definition won't be generated.

## Database Constraints

- Do not add `UNIQUE` constraints on `users.email` or `users.phone`.  Uniqueness for
  contact fields is enforced at the application layer, not the database schema.

## Index Style

- Always use the SeaORM builder API (`Index::create().name(...).table(...).col(...).to_owned()`)
  when creating indexes in migrations.
- Extract each index name into a `const` at the top of the migration file so the string
  is shared by `up` and `down` without duplication.

## Verification Commands

Before claiming work is complete, run these commands:

- `cargo fmt --all -- --check` — format check
- `cargo clippy --all-targets -- -D warnings` — lint check
- `cargo test --workspace -r` — full test suite (release mode, requires PostgreSQL)

## Code Style Conventions

- **`tap` / `Pipe`**: Use `tap` for side-effect chaining (especially span field recording). Use `pipe` for method chaining.
- **`linkme::distributed_slice`**: Used for extensible registries (background workers). Register new implementations via `#[distributed_slice]`.
- **`moka`**: Primary caching strategy. Every state manager (keyboxes, credentials, revoked JWTs, application configs) uses a `moka::future::Cache` with appropriate TTL/capacity.
- **`#[tracing::instrument]`**: Required on all endpoint handlers. Use `skip(...)` for parameters that don't need tracing and `fields(...)` for structured span data.
- **Tracing field shorthand**: In `info!`, `error!`, `warn!`, `debug!` macros, prefer `%field` over `field = %field` when the field name matches the variable name. For example, write `info!(%application_id, "msg")` instead of `info!(application_id = %application_id, "msg")`.
- **`sqids`**: All resource IDs in URLs are Sqid-encoded. Convert to/from `Uuid` via `oceaniam_common::sqid::Sqid`.
- **`garde` + `axum-valid`**: Use `#[derive(Validate)]` with `garde` annotations for request body validation. Apply via `Garde` extractor.
- **`thiserror` + `oceaniam_common::error::Error`**: Use `Error::with_code(StatusCode, msg)` for typed HTTP errors. Avoid raw `StatusCode` returns.
- **Response types**: Use `oceaniam_api::ApiResponse<T>` for success, `ApiResponseWithHeader<T>` for responses with custom headers (cookies), and `RestResult<T>` / `WithHeaderRestResult<T>` as return type aliases.
  Array-typed responses (`Vec<T>`) must always be wrapped in `PagedResponse<T>` — even when
  the endpoint returns all items without pagination. Use `PagedResponse::with_entire(...)` for
  unpaginated lists.
- **`#[allow(unused)]`**: Used as a WIP marker for modules or structs that are not yet wired up to the endpoint layer.
- **`crate::conversion` module**: Model→VO 转换统一放在 `crates/oceaniam/src/conversion/` 中，使用普通函数而非 `From` trait impl。`oceaniam-vo` 只包含纯数据类型（struct + derive），不包含业务逻辑或数据库依赖。新增端点需要 VO 转换时，优先在该模块中添加对应的转换函数。
- **Transaction wrapping**: Multi-write operations (e.g., create entity + create related resources, delete-all + insert-many) must be wrapped in an explicit `database.begin()` / `tx.commit()` pair. Pass `&tx` as the connection parameter to all helpers within the transaction scope. If a helper does not accept a generic connection, expose a `_in_tx` variant (see below).
- **State layer `_in_tx` pattern**: State managers that perform DB writes must expose two variants for multi-step operations: `method(&self, ..., &self.database)` (convenience, uses the raw connection) and `method_in_tx(&self, ..., transaction)` (for callers that need to share a transaction). Existing examples: `update_password` / `update_password_in_tx` (`state/credentials.rs`), `create_keybox` / `create_keybox_in_tx` (`state/keybox.rs`), `set_pass` / `set_pass_in_tx` (`state/challenge.rs`).
- **`lib.rs` as pure re-exports**: Implementation logic must live in named sub-modules; `lib.rs` should contain only `mod` declarations and `pub use` re-exports. This keeps the public API surface explicit and avoids internal implementation details leaking into the crate root. See `oceaniam-credential/src/lib.rs` / `vault.rs` for the canonical example.

## Testing Infrastructure

- Integration tests live in `crates/oceaniam/tests/` and use `#[tokio::test]` with `reqwest`.
- Use `tests/support/mod.rs::spawn_app_with_isolated_schema()` to set up tests:
  - Creates a unique PostgreSQL schema per test (`test_schema_{uuid}`).
  - Runs all migrations inside that schema.
  - Starts the Axum server on a random port (`0.0.0.0:0`).
  - Auto-cleans the schema on `Drop` via a dedicated tokio runtime thread.
- Database connection defaults to `postgresql://postgres:postgres@localhost:5432/postgres`.
- Root password can be set via `MIGRATION_DEFAULT_ROOT_PASSWORD` environment variable.
- When running `oceaniam-keybox` crate tests, use `-r` (release mode) — they compile
  and execute significantly faster than debug builds.

## Migration Discipline

- Migrations are always linear and append-only.  Never delete, rename, or modify a migration
  file that has already been committed — future migrations must compensate instead.
- Every migration must be idempotent (usable in both fresh installs and incremental rollouts).
- After writing a migration and running `up`, run `just gen-entities` from the workspace root to
  regenerate the SeaORM entity models so they reflect the current database schema.
- When creating a new migration, use `sea-orm-cli migrate generate <name>` to scaffold it.
  Run the command inside the `crates` directory (`backend/crates/`) so the scaffolding lands in
  the correct migration crate.
  If `sea-orm-cli` is not installed, run `cargo install sea-orm-cli` first.  If the
  installation fails, stop and inform the user — do not proceed without a proper
  migration scaffold.
