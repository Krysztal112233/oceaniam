# Authentication Error Codes

This document describes the HTTP status code conventions used throughout
the OceanIAM API.

## Design Goals

- **Prevent information leakage**: Error responses must not reveal whether a specific user, email, or phone is registered.
- **Support dual auth paths**: Some endpoints accept either a platform JWT or an application secret. The error code scheme must allow the `Either` guard combinator to distinguish "try the other path" from "hard failure".

## Status Code Reference

### 203 Non-Authoritative Information — Switch Auth Method

Returned by `extract_bearer_token` (in `util/jwt.rs`) when the `Authorization` header is absent. This is **not** a success or error code — it is a sentinel consumed by `AdminJwtOrApplicationSecretGuard` (`Either<PlatformAuthGuard, MatchedApplicationSecretGuard>`): the `Either` combinator forwards 203 to the second branch (application secret auth) and treats all other codes as a hard failure of the first branch.

### 400 Bad Request — Invalid Token or Request

All JWT validation failures (expired token, bad signature, wrong issuer or audience, revoked JTI, missing `kid`, malformed header) return 400 without distinguishing the specific cause. This prevents attackers from probing whether a token expired or was forged.

Also used for malformed request paths, missing search parameters, or unparseable challenge payloads — cases where the request itself is wrong and retrying with the same input will not help.

### 401 Unauthorized — Authentication Required

- Missing or invalid `X-OceanIAM-Application-Secret` header.
- Platform login with invalid credentials (user not found or wrong password — both return 401 to prevent enumeration).
- Application-scoped login with invalid credentials (user not found or wrong password — both return 401 to prevent enumeration).
- TOTP / MFA challenge verification failure.

These are cases where valid credentials exist but were not provided, or where the provided credentials are provably wrong (as opposed to "looks wrong", which falls under 400). The caller should re-authenticate.

### 403 Forbidden — Insufficient Permissions

The request is authenticated but the subject lacks the required ORBAC permission (`PlatformPermissionGuard`) or the application secret does not belong to the targeted application (`MatchedApplicationSecretGuard`).

### 404 Not Found — Resource Does Not Exist

Standard resource-not-found semantics for tenants, applications, users, administrators, challenges, secrets, and application keys.

### 409 Conflict — Duplicate Resource

Returned when creating an administrator with a name that already exists.

### 500 Internal Server Error — Unexpected Failure

- Infrastructure failures: keybox, credential vault, bloom filter, JWT library, Argon2 parameter construction.
- Database connection or constraint errors.
