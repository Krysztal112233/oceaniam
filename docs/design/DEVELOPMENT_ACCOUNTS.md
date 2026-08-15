# Development Accounts

Development accounts ("dev accounts") are time-limited application users intended
for automated testing by third-party applications. An application (or a platform
administrator) creates one through the ordinary application-user creation API; the
account behaves exactly like a normal application user, and the system permanently
deletes it after the requested TTL.

## Decisions (confirmed with the project owner)

| Topic | Decision |
| --- | --- |
| Granularity | A normal user/subject inside an **existing** application, carrying an expiry timestamp. No temporary tenants/applications. |
| API contract | `POST /tenants/{tenant}/applications/{app}/users`. Absence of `development` creates a permanent user; `development: {}` creates a dev user with the default TTL. |
| Caller & auth | `AdminJwtOrApplicationSecretGuard` — an application holding an `app_` secret self-serves users for itself; platform admins may also create them. No unauthenticated path. |
| TTL semantics | `development.ttl_seconds` is optional. Default **3600** (1 hour) when omitted inside `development`. Valid range is `1..=2147483647`, matching pgmq's PostgreSQL `integer` delay. No renewal/extension (for now). |
| Discriminator | Plain column `subjects.expires_at TIMESTAMPTZ NULL`; `NULL` means a normal account. No new `subject_type_enum` variant. |
| Expiry execution | **pgmq delayed message + in-API-process consumer** (see below). No periodic sweep. |
| Lazy rejection | Sign-in and token-refresh reject accounts whose `expires_at <= now()`. |
| Poison/idempotency | Deleting an already-gone account is success. Transient errors are left for visibility-timeout redelivery. `read_ct >= 10` → archive to `pgmq.a_dev_account_expiration` (dead letter). |
| Guardrails | No per-application toggle or rate limiting (for now). |
| Token lifetime | Access tokens issued before expiry remain valid until their own `exp`; token clamping to `expires_at` is explicitly out of scope. Operators should keep access-token TTLs short for dev-account-heavy applications. |
| Dead letters | A dead-lettered expiration message (`pgmq.a_dev_account_expiration`) means the account is **never deleted** (it is still lazy-rejected at sign-in/refresh). Operators should monitor the archive table. |
| SDK | Rust SDK updated through `create_application_user`; Dart SDK untouched. |

## Data model

- Migration (handwritten Rust module, repo convention):
  - `ALTER TABLE subjects ADD COLUMN expires_at TIMESTAMPTZ NULL;`
  - `CREATE EXTENSION IF NOT EXISTS pgmq;`
  - `SELECT pgmq.create('dev_account_expiration');`
- No index on `expires_at` (no sweep; lazy checks are by primary key).
- `down`: drop the column, drop the queue guarded by an existence check on `pgmq.meta`
  (so a partially-applied `up` does not break rollback).

## Creation flow

The ordinary application-user creation endpoint is the only creation endpoint:

```text
POST /tenants/{tenant}/applications/{app}/users
Guard: AdminJwtOrApplicationSecretGuard
```

The request keeps normal user fields at the top level and nests development options:

```json
{
  "email": "dev@example.com",
  "password": "DevPassword123!",
  "development": {
    "ttl_seconds": 3600
  }
}
```

The `development` field has presence-sensitive semantics:

- field absent: create a permanent user with `subjects.expires_at = NULL`, enqueue no
  pgmq message, and write `CreateApplicationUserPayload`;
- `"development": {}`: create a development account with the default 3600-second TTL;
- `"development": {"ttl_seconds": N}`: create a development account with the requested
  validated TTL.

For a development account, inside the same database transaction as account creation
(the `create_user_in_tx` path):

1. Create `credentials` → `subjects` → `users` rows, reusing the existing creation
   ordering.
2. Set `subjects.expires_at = now() + ttl`.
3. Enqueue the expiry message atomically via raw SQL on the same transaction:

   ```sql
   SELECT pgmq.send('dev_account_expiration',
                    '{"subject_id": "...", "application_id": "..."}'::jsonb,
                    <ttl_seconds>);
   ```

   `application_id` is included because deletion needs it for cache eviction.

The unified creation response has the normal application-user wire shape. It adds
`expires_at` only for development accounts; permanent-user responses omit the field.
Development creation writes `CreateDevAccountPayload` after the transaction commits.

## Expiry execution (pgmq)

The pgmq message delay **is** the timer: the message becomes visible exactly at
`expires_at`.

- **Consumer location**: inside the API process (not the `oceaniam-worker`
  container), because deletion reuses `ApplicationUsers::delete_user_in_tx`,
  which performs moka cache eviction. The worker process has no state managers
  and the credential cache TTL is 30 minutes — worker-side deletion would leave
  a stale-auth window.
- **Loop**: a tokio task spawned at API bootstrap.
  - `pgmq.read('dev_account_expiration', vt = 60, qty = 10)` via raw SQL
    through SeaORM (`Statement::from_sql_and_values`; precedent:
    `oceaniam-database/src/helper/trend.rs`). No `pgmq` crate dependency.
  - Success → `delete_user_in_tx` → `pgmq.delete(queue, msg_id)`.
  - Subject already gone (manual deletion raced the timer) → treat as success
    (idempotent), `pgmq.delete`.
  - Transient error (DB jitter etc.) → no ack; the message reappears after VT.
  - `read_ct >= 10` → `pgmq.archive` (dead letter), do not process.
  - Empty read / read error → interruptible sleep (~1s).
  - **Never cancel an in-flight `pgmq.read`**: pgmq claims the message
    server-side the instant the read executes (`read_ct += 1`, VT set).
    Cancelling burns a retry and delays redelivery by one VT. Bound shutdown
    latency via the interruptible idle sleep only.
  - Multi-replica safe: `pgmq.read` uses `FOR UPDATE SKIP LOCKED`.
- Deletion writes an audit event distinct from manual deletion
  (`DevAccountExpiredPayload`).

## Lazy rejection

Sign-in and token-refresh paths check `subjects.expires_at <= now()` and reject
(401, per `docs/design/ERROR_CODES.md` conventions). This covers the window
between expiry and message consumption.

## Infrastructure

- `Dockerfile` `database` target: compile pgmq **v1.12.0** from source, following
  the existing pg_duckdb build pattern. pgmq does **not** need
  `shared_preload_libraries`.
- `docker/postgres/initdb/01-enable-pgmq.sql`:
  `CREATE EXTENSION IF NOT EXISTS pgmq;`
- CI builds the database image for the test job, so integration tests get pgmq
  automatically.

## SDK

- Rust SDK (`sdk/rust`): `create_application_user(...)` accepts
  `CreateApplicationUserRequest.development` and returns the unified creation response.
- Dart SDK: deliberately not updated.

## Testing

Integration tests (schema-isolation harness, `backend/crates/oceaniam/tests`):

1. Creating a permanent user without `development` leaves `expires_at` null and
   enqueues no expiration message.
2. Creating a dev account through `POST .../users` succeeds, sign-in works, and the response
   `expires_at` matches the persisted timestamp.
3. Forcing `expires_at` into the past via direct DB update makes sign-in return 401.
4. A delayed message lands in `pgmq.q_dev_account_expiration` with the expected payload and
   delay.
5. Invoking the consumer directly deletes the account (including cache eviction), while repeated
   processing is a no-op success.
6. Token refresh succeeds before expiry and returns 401 after expiry.
7. Missing authentication is rejected, and an application secret cannot target a different
   application.
8. An application-secret caller can create a development account for its own application.
9. Nested TTL validation rejects zero and values above PostgreSQL `integer`; `development: {}`
   uses 3600 seconds, while a non-default custom TTL controls both `expires_at` and pgmq delay.
