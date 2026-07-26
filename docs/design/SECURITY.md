# Security Design

Due to the special nature of this project, various security aspects need to be addressed. These security designs include but are not limited to:

- Storage and management of user credentials
- Storage and management of system user configurations
- Storage and management of application secrets
- Password hashing strength design
- ...and more

## Password Hashing

This design follows the OWASP [Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) recommendations.

Due to the unique nature of password storage—passwords cannot be changed quickly in bulk and are vulnerable to offline attacks once leaked—choosing a reliable password hashing algorithm is essential. In this system design, argon2id is selected as the password hashing algorithm, which provides protection against side-channel attacks and GPU-based attacks.

> [!NOTE]
> Furthermore, slowing down the attacker's cracking speed is crucial.
>
> However, once an attacker has acquired stored password hashes, they are
> always able to brute force hashes offline. Defenders can slow down offline
> attacks by selecting hash algorithms that are as resource intensive as
> possible.

For argon2id, OWASP recommends the following parameters:

- m=12288 (12 MiB), t=3, p=1
- m=9216 (9 MiB), t=4, p=1
- m=7168 (7 MiB), t=5, p=1

These parameters can achieve the same security level, but the trade-off lies in the consumption between CPU and memory.

Considering the request volume, this system chooses to consume more memory to save CPU resources.

## Application Secrets

Application Secrets are generated as `app_` followed by 32 random ASCII alphanumeric characters.
The full value is revealed only by the create response; list and get responses expose only the
existing eight-character display prefix followed by `...`.

The database stores no plaintext Application Secret. It stores a 12-character lookup prefix, a
32-byte HMAC-SHA-256 verifier, and the HMAC key version. The verifier uses the domain-separated
input `oceaniam/application-secret/v1\0 || full_secret`. Authentication queries all non-revoked
records sharing the prefix and compares each verifier in constant time.

Application Secret HMAC keys are independent from the system Master Key. Configuration contains a
positive `current_version` and a version-to-key map; every key is a non-zero 32-byte value generated
with `openssl rand -hex 32`. The service refuses to start when a non-revoked database record
references a key version absent from configuration. Revoked rows are excluded from startup key
validation because they cannot authenticate and do not need verifier upgrades.

To rotate keys, add the new version without removing old versions and make it current. Successful
authentication with an old version opportunistically upgrades that active record. Remove an old key
only after every active record has either been upgraded or revoked; retained revoked rows do not
require retired keys. Authentication results are not process-cached, so deletion and revocation take
effect across instances on the next request.

The verifier-only migration requires a maintenance window: stop every old secret-writing backend,
verify a restorable database backup, run the migration, and then deploy the verifier-aware runtime.
Rollback means restoring that backup because plaintext cannot be reconstructed. The migration
specifically reads `OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__1`, even when runtime configuration
comes from TOML, and that value must exactly match the runtime version-1 key.
