# OceanIAM SDK — Agent Instructions

## Sqid Type Handling

`Sqid` (`oceaniam_vo::sqid::Sqid`) is a `#[repr(transparent)]` newtype over `String`. In the SDK,
it is used solely as a route parameter — the value is always put directly into the URL path without
any sqid decoding or encoding.

Therefore, **all SDK method signatures should accept `&str` instead of `impl Into<Sqid>`** for ID
parameters. Callers can pass a plain string or a `&Sqid` (via `Deref<Target=str>`).
No `.parse()` or `Sqid` construction is needed at the SDK boundary.
