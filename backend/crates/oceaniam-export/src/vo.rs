pub mod administrators;
pub mod applications;
pub mod auth;
pub mod pagination;
pub mod sqid;
pub mod statistics;
pub mod tenants;

/// How an authentication token should be delivered back to the client.
///
/// This is typically chosen by the endpoint that issues or refreshes tokens.
///
/// The dispatch method can be controlled by the request header
/// `X-OceanIAM-Token-Dispatch`.
///
/// - Accepted values: `cookie`, `json`, `both` (case-insensitive; whitespace is ignored)
/// - Default: missing/invalid header falls back to `both`
///
/// Common use-cases:
/// - Browser apps often prefer `Cookie` so the token is automatically attached
///   to subsequent requests (and can be protected with `HttpOnly`, `Secure`,
///   `SameSite`, etc.).
/// - API clients (mobile/desktop/CLI/SDK) often prefer `Json` so they can
///   manage the token explicitly (e.g., attach it as an `Authorization: Bearer`
///   header).
/// - During migrations or when you need to support both kinds of clients,
///   `Both` can be used to send the token in both places.
#[derive(Debug, Clone, ts_rs::TS)]
#[ts(export)]
pub enum TokenDispatchMethod {
    /// Deliver the token via an HTTP cookie (e.g., `Set-Cookie`).
    ///
    /// Useful when:
    /// - The client is a browser and you want session-like behavior.
    /// - You want to use cookie attributes (`HttpOnly`, `Secure`, `SameSite`)
    ///   to improve safety ergonomics.
    Cookie,

    /// Deliver the token via JSON response body.
    ///
    /// Useful when:
    /// - The client is not a browser (mobile, desktop, CLI, server-to-server).
    /// - The client wants to decide how/where to persist the token.
    Json,

    /// Deliver the token both via cookie and JSON.
    ///
    /// Useful when:
    /// - You have mixed clients and want a single response format.
    /// - You are migrating between `Cookie` and `Json` based auth flows.
    Both,
}
