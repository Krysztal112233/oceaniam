# `oceaniam-database::helper`

Per-entity database access helpers. Each module defines a trait with default method implementations
and a blanket `impl Trait for Entity {}` that enables `Entity::method(...)` calls.

```rust,ignore
// Example (not executable):
// use crate::helper::subjects::SubjectsHelper;
// Subjects::create_subjects(id, app_id, typ, &db).await?;
```

All methods accept `&impl SafeTransactionConnectionTrait` — pass either a `&DatabaseConnection` or a
`&DatabaseTransaction` interchangeably, allowing callers to compose operations atomically without
changing helper signatures.

- `SafeTransactionConnectionTrait` — unifies `ConnectionTrait + TransactionTrait`
- `PagedSelect` — pagination combinator on `Select<T>`
- `PagedExecutor` — fetches items + total count into `PagedResponse`
- `exactly_one_returned` — collapses a primary-key-filtered `exec_with_returning` result into
  one row, keeping the `debug_assert!(len <= 1)` invariant in one place and mapping an empty
  result to the caller's not-found error
- `pgmq` — free-function wrappers over the `pgmq` extension (`send`/`read`/`delete`/`archive`);
  pgmq has no SeaORM entity, so this module is the sanctioned exception to the per-entity
  trait shape below

## Database access rules

**Rule 1 (business packages).** Business packages (`oceaniam`, `oceaniam-oidc` `src/`) MUST NOT
use SeaORM query builders, entity queries, or raw `Statement` SQL directly; all database access
goes through the State facade or `oceaniam-database` helpers. The only exception is
`backend/crates/oceaniam/tests/`: seeding and asserting state directly with builders is the
legitimate job of tests.

**Rule 2 (helper crate idioms).** Single-row operations inside this crate MUST use the canonical
patterns:

- Single-row **read**: `find_by_id()/filter + [.lock_exclusive()/.lock_shared()] + .one() +
  ok_or_else(404)`.
- Single-row **write**: primary-key-filtered conditional `update_many` +
  `exec_with_returning` + `exactly_one_returned`.

Exceptions: multi-table row operations (ordered lock chains such as
`oidc_authorization_entry.rs`; multi-table writes in one transaction such as
`applications_secrets::create_binding_pair`, `users::create_user`).

**Rule 3 (single time authority).** Any timestamp persisted to the database, or evaluated by a
database predicate, MUST originate from the database clock (`clock_timestamp()`, sampled via
`db_clock_now` or used inline in SQL). The application clock (`Utc::now()` and friends) is
reserved for logs, tracing/metrics, in-process state (e.g. anti-replay cache TTLs), and
presentational output.

## Convention

Method bodies MUST be written as default implementations inside the trait declaration.
The `impl Trait for Entity {}` block MUST remain empty — it exists only to register the
trait as an implementor on the entity.

```rust,ignore
// pub trait XxxHelper {
//     async fn method(...) -> Result<...> {
//         // method body here
//     }
// }
//
// impl XxxHelper for XxxEntity {}
```

This differs from the standard Rust pattern (trait with signatures + separate impl block).
Reasons: reduces boilerplate when adding new methods, and keeps each helper in a single
readable block. `role_permissions.rs` initially followed the wrong pattern; it has been
corrected.
