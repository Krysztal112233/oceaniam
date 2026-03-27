# Backend Agent Instructions

These instructions apply to work performed inside the `backend` project.

## Authorization Model

- This backend implements an ORBAC-based IAM model.
- When reasoning about authorization behavior, prefer ORBAC terminology instead of reducing the system to a simple RBAC-style user-role-permission mapping.
- `organization`: the organizational boundary where an authorization rule applies.
- `role`: the responsibility or function a subject holds within an organization.
- `activity`: the business action governed by an authorization rule.
- `view`: the resource view or resource set targeted by an authorization rule.
- `context`: the condition or situation under which an authorization rule becomes effective.
- If a change introduces or modifies authorization semantics, make sure the corresponding `organization`, `role`, `activity`, `view`, and `context` meaning is clear before implementation.

## Change Scope

- Keep every change as small as possible.
- Limit edits to the minimum required to complete the current task.
- Do not mix unrelated refactors, opportunistic cleanup, or cross-module changes into the same task.

## Existing Backend Changes

- If the workspace already contains a significant amount of pending backend-related changes, warn the user before continuing.
- Ask the user to save or explicitly confirm those existing changes first.
- Do not proceed with further backend work until the user has explicitly agreed.

## Unit Tests

- Any unit test written by the agent must include `// NOTE: AI-generated test` immediately above the test.

## Design Responsibility

- Focus on helping the user complete tedious, repetitive, or execution-heavy work.
- Do not replace the user in system design, architecture design, or major solution-shaping decisions.
- If the user is asking for system design work, instruct them to enter Plan Mode first, complete the plan, and only then move on to implementation.

## Backend and Frontend Isolation

- When making backend changes, do not modify files under `frontend/`.
- Treat `backend` and `frontend` as independently developed projects unless the user explicitly requests coordinated cross-project work.

## Endpoint Documentation Consistency

- When helping implement or modify endpoints in `crates/oceaniam/src/endpoints/`, update the corresponding `utoipa` annotations in the same change.
- If an endpoint's path, method, parameters, request body, response body, tags, or security behavior changes, update the related `#[utoipa::path(...)]` attributes so the OpenAPI description stays accurate.
- If a newly added or modified endpoint is exposed through the frontend SDK, update `frontend/packages/sdk/src/client.ts` in the same change so the SDK client stays in sync with the backend API surface.
