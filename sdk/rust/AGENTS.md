# OceanIAM SDK — Agent Instructions

## Sqid Type Handling

`Sqid` (`oceaniam_common::sqid::Sqid`) is a `#[repr(transparent)]` newtype over `String`. In the SDK,
it is used solely as a route parameter — the value is always put directly into the URL path without
any sqid decoding or encoding.

Therefore, **all SDK method signatures should accept `&str` instead of `impl Into<Sqid>`** for ID
parameters. Callers can pass a plain string or a `&Sqid` (via `Deref<Target=str>`).
No `.parse()` or `Sqid` construction is needed at the SDK boundary.

## Verification Commands

Before claiming work is complete, run these commands:

- `cargo fmt --all -- --check` — format check
- `cargo clippy --all-targets -- -D warnings` — lint check
- `cargo test --workspace -r` — full test suite (release mode)

## Dependency Conventions

- `oceaniam-vo` is a local path dependency (`../../backend/crates/oceaniam-vo`). When the backend VO types change, the SDK may need corresponding updates.
- Avoid adding `oceaniam-common` or other backend-internal crates as direct dependencies — keep the SDK boundary thin.
- Error handling uses `snafu` — wrap external errors with `Snafu` derive macros.

## Code Style

- Match the backend code style conventions where applicable: `tap` for side-effect chaining, `tracing` for logging, `serde` for (de)serialization.
