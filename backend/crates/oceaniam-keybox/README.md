# oceaniam-keybox

**Tenant-scoped cryptographic key management for OceanIAM.**

`oceaniam-keybox` manages RSA key pairs used for signing and verifying JWTs. It provides the `KeyBox` — an in-memory container that holds all cryptographic keys for a tenant, supports the full key lifecycle (pending → active → retired, plus revocation), and can serialize itself into a JWK Set for public key distribution.

## Key Lifecycle

```text
  Pending ----> Active ----> Retired
     |            |            |
     +----+-------+------------+----> Revoked
```

### Allowed Transitions

| From    | To      | Condition                            |
| ------- | ------- | ------------------------------------ |
| Pending | Active  | `activated_at` reached               |
| Pending | Retired | Skipped activation                   |
| Pending | Revoked | Manual revocation                    |
| Active  | Retired | `retired_at` or `expires_at` reached |
| Active  | Revoked | Manual revocation                    |
| Retired | Revoked | Manual revocation                    |

### Prohibited Transitions

| From    | To                         |
| ------- | -------------------------- |
| Retired | Pending / Active           |
| Revoked | Pending / Active / Retired |

### Timeline Fields

- `activated_at` — key becomes usable for signing
- `retired_at` — key is no longer used for new signatures (may still verify existing tokens)
- `expires_at` — key is fully expired
