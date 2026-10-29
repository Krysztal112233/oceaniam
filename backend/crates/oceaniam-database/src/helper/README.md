# `oceaniam-database::helper`

Per-entity database access helpers: each module is a trait of default-implemented methods plus an
empty `impl Trait for Entity {}` (registration only), enabling `Entity::method(...)` calls.

All methods take `&impl SafeTransactionConnectionTrait` — pass `&DatabaseConnection` or
`&DatabaseTransaction` interchangeably, so callers compose atomically without signature changes.

Shared facilities:

- `SafeTransactionConnectionTrait` — unifies `ConnectionTrait + TransactionTrait`
- `PagedSelect` / `PagedExecutor` — pagination into `PagedResponse`
- `exactly_one_returned` — collapses a primary-key-filtered `exec_with_returning` into one row,
  holding the `debug_assert!(len <= 1)` invariant and mapping empty to the caller's not-found
- `pgmq` and `worker_jobs` — free functions over extension-owned queues, which have no
  SeaORM entity; these are the sanctioned exceptions to the per-entity trait shape

## Database access rules

**Rule 1 (business packages).** Business packages (`oceaniam`, `oceaniam-oidc` `src/`) MUST NOT
use SeaORM query builders, entity queries, or raw `Statement` directly; all database access goes
through the State facade or these helpers. Exception: `backend/crates/oceaniam/tests/`, where
seeding and asserting state with builders is the legitimate job of tests.

**Rule 2 (helper idioms).** Single-row operations use the canonical shapes:

- **Read**: `find_by_id()/filter + [.lock_exclusive()/.lock_shared()] + .one() + ok_or_else(404)`
- **Write**: primary-key-filtered conditional `update_many` + `exec_with_returning` +
  `exactly_one_returned`

Exceptions: multi-table operations (ordered lock chains such as `oidc_authorization_entry.rs`;
multi-table writes in one transaction such as `applications_secrets::create_binding_pair`,
`users::create_user`).

**Rule 3 (single time authority).** Any timestamp persisted to the database, or evaluated by a
database predicate, MUST originate from the database clock (`clock_timestamp()`, sampled via
`db_clock_now` or used inline in SQL). The application clock (`Utc::now()` and friends) is
reserved for logs, tracing/metrics, in-process state (e.g. anti-replay cache TTLs), and
presentational output.

## Convention

Method bodies live in the trait as default implementations; the `impl Trait for Entity {}` block
stays empty.

Conditional updates (`Entity::update_many()`) are written predicate-first: every `filter` before
any `col_expr` / `set` — which rows qualify comes before what changes.
