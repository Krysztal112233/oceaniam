# `oceaniam-database::helper`

Per-entity database access helpers. Each module defines a trait with default method implementations
and a blanket `impl Trait for Entity {}` that enables `Entity::method(...)` calls.

```rust
use crate::helper::subjects::SubjectsHelper;
Subjects::create_subjects(id, app_id, typ, &db).await?;
```

All methods accept `&impl SafeTransactionConnectionTrait` — pass either a `&DatabaseConnection` or a
`&DatabaseTransaction` interchangeably, allowing callers to compose operations atomically without
changing helper signatures.

- `SafeTransactionConnectionTrait` — unifies `ConnectionTrait + TransactionTrait`
- `PagedSelect` — pagination combinator on `Select<T>`
- `PagedExecutor` — fetches items + total count into `PagedResponse`

## Convention

Method bodies MUST be written as default implementations inside the trait declaration.
The `impl Trait for Entity {}` block MUST remain empty — it exists only to register the
trait as an implementor on the entity.

```rust
pub trait XxxHelper {
    async fn method(...) -> Result<...> {
        // method body here
    }
}

impl XxxHelper for XxxEntity {}
```

This differs from the standard Rust pattern (trait with signatures + separate impl block).
Reasons: reduces boilerplate when adding new methods, and keeps each helper in a single
readable block. `role_permissions.rs` initially followed the wrong pattern; it has been
corrected.
