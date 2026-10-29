# OceanIAM Worker

Every task executes from the PGMQ `worker_jobs` queue. Optional UTC cron producers only
enqueue triggers; they never call business handlers directly.

## Registration

Implement `oceaniam_worker_runtime::Worker<WorkerContext>` with `name()` and `run()`, then
register a factory with `#[distributed_slice(REGISTERED_WORKERS)]`:

```rust,ignore
OceaniamWorkerRegistration {
    worker: Arc::new(MyWorker),
    cron: Some("0 0 */6 * * *"), // None for message-only tasks
}
```

The existing key-rotation and expired-OIDC cleanup schedules are unchanged. The development-account
expiration consumer remains in the API process because it also invalidates local caches.

## Publishing and deployment

- Manual/event triggers use
  `oceaniam_database::helper::worker_jobs::enqueue(name, delay_seconds, &connection)`.
  Pass a business transaction to publish atomically with its writes.
- The message format is `{"worker":"key_rotation"}`; unknown or malformed triggers are archived.
- Run one worker process. Cron waits for the next UTC occurrence and enqueues directly;
  there is no distributed publication deduplication or persistent schedule cursor. Missed
  occurrences during downtime are not replayed. Failed enqueue attempts retry after five seconds.
- Apply migrations before starting producers or consumers. Stop old cron-only worker binaries
  before deploying this version: they execute handlers directly instead of using the queue.
  No `pg_cron` extension is needed.

## Execution contract

One polling loop runs up to four handlers concurrently per process. Claims have a 120-second
visibility timeout, renewed every 30 seconds. Successful handlers ack their message; failures
leave it for redelivery. Ten failed/crashed claims exhaust the budget and move the trigger to
`pgmq.a_worker_jobs`, with an operator-attention warning. Replaying archived jobs is an operator
action, not automatic. Poll errors back off to 60 seconds.

Delivery is **at least once**, not exactly once. Handlers must tolerate repeated or overlapping
execution. Key rotation serializes each tenant using a transaction-scoped advisory lock and
commits key changes with its audit. Lease renewals, acknowledgements and archiving are fenced by
the claim's read count, so a stale consumer cannot remove a newer claim.

Shutdown stops cron publication and new reads, then waits for running handlers and their acks.
An in-flight read is not cancelled, and existing handlers keep renewing leases during drain.
