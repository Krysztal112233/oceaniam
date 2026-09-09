# Migration Discipline & Rollback Policy

OceanIAM migrations are **forward-only in production**: a release migrates up,
and rolling back a release means restoring the pre-upgrade backup or shipping a
new compensating migration. `down()` implementations exist to keep development
workflows (`sea-orm-cli migrate refresh/reset/down`) usable, not as a
production rollback mechanism.

## Rules

1. **Append-only history.** Never delete, rename, or modify a committed
   migration file — future migrations must compensate instead. Sole exception:
   a committed migration's `down` side may be repaired to correct development
   rollback behavior. A down-only change does not affect normal `up` execution
   or SeaORM migration validation because SeaORM records no checksum, but it
   does change how databases that already applied the migration behave during
   `down`, `reset`, or `refresh`. Document the reason and impact, and see
   `backend/AGENTS.md` → Migration Discipline.
2. **New migrations must implement `down()`** that exactly reverses `up()`
   (reverse order for multi-step changes). If a migration is genuinely
   irreversible, make `down()` return an error whose message starts with
   `irreversible:` and explains why. CI rejects PRs whose newly added
   migrations have an empty `down()` without such a marker.
3. **One concern per migration.** Do not bundle unrelated changes into one
   migration. Historical counter-example: `m20260614_082902_envelope_encrypt_keys/up.sql`
   also purged the dead `auth.totp` configuration key (later cleaned up by
   `m20260902_020917_purge_application_configuration_totp`).
4. **Idempotent and additive-first.** Prefer `IF NOT EXISTS` /
   `ON CONFLICT DO NOTHING`; split renames into add → migrate → drop steps;
   avoid mixing DDL and DML in one migration (Postgres DDL is transactional —
   keep rollback boundaries clean).
5. **Scaffold with the CLI**: run `sea-orm-cli migrate generate <name>` inside
   `backend/crates/`, and after applying `up`, run `just gen-entities` from the
   workspace root to regenerate the SeaORM entity models.

## Known irreversible migrations

Rolling back through any of these requires restoring from a backup.

### Empty `down()` (enum value additions; Postgres enums are append-only in practice)

- `m20260105_141454_add_rs384_rs512`
- `m20260105_144550_add_ps256_ps384_ps512`
- `m20260312_135205_alter_audit_type_refresh_jwt`
- `m20260321_091710_alter_audit_type_application_configuration`
- `m20260324_101500_alter_audit_type_patch_application`
- `m20260326_112803_alter_audit_type_tenant_admin_management`
- `m20260327_202624_alter_audit_type_patch_administrator`
- `m20260426_141557_create_audit_type_for_challenges`
- `m20260428_015000_alter_audit_type_verify_challenge`
- `m20260511_133000_alter_audit_type_rotate_revoke_key`
- `m20260615_081707_add_email_totp_challenge_factor`
- `m20260620_085000_alter_audit_type_bind_unbind_secret`

### Explicitly irreversible

- `m20260614_082902_envelope_encrypt_keys` — `up` overwrites plaintext PEMs with
  envelope-encrypted ciphertexts that SQL cannot decrypt. Its historical
  `down.sql` contained unrelated, destructive statements and was removed on
  2026-09-02 under the down-side repair exception; `down()` now returns an
  `irreversible:` error.
- `m20260726_142405_hash_application_secrets` — one-way HMAC hashing of stored
  application secrets cannot be reversed.

## Backup & restore (production rollback)

Take a snapshot **before every upgrade**:

```bash
pg_dump --format=custom --file oceaniam-pre-upgrade.dump "$DATABASE_URL"
```

To roll back a release:

1. Stop the backend and worker services.
2. Restore the snapshot into a fresh database:
   `pg_restore --clean --if-exists --dbname "$DATABASE_URL" oceaniam-pre-upgrade.dump`
3. Redeploy the previous `OCEANIAM_TAG` images.
