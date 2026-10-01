//! Transport types for the OIDC authorization login form.
//!
//! The login form shares the Authorization Endpoint's transport boundary: one bounded
//! `application/x-www-form-urlencoded` payload, deserialized lossily with empty values treated
//! as omission. This module deliberately does not authenticate anything; HTTP callers must
//! establish cookie binding, CSRF, transaction validity, and live registration trust before
//! using these values.

use serde::Deserialize;

/// Deserialized login form values awaiting binding and credential checks.
///
/// This type deliberately omits `Debug`: it carries the submitted password. Unknown fields are
/// discarded during deserialization; missing or empty fields stay omitted so callers can apply
/// their uniform rejection semantics.
#[derive(Deserialize)]
pub struct ParsedLoginForm {
    identifier: Option<String>,
    password: Option<String>,
    csrf: Option<String>,
}

impl ParsedLoginForm {
    /// Returns the single identifier (email or phone) exactly as submitted.
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Returns the submitted password without any trimming or normalization.
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Returns the submitted CSRF secret exactly as submitted.
    pub fn csrf(&self) -> Option<&str> {
        self.csrf.as_deref()
    }
}

/// Deserialized challenge form values awaiting binding and OTP checks.
///
/// This type deliberately omits `Debug`: it carries the submitted one-time code. Unknown fields
/// are discarded during deserialization; missing or empty fields stay omitted so callers can
/// apply their uniform rejection semantics.
#[derive(Deserialize)]
pub struct ParsedChallengeForm {
    code: Option<String>,
    csrf: Option<String>,
}

impl ParsedChallengeForm {
    /// Returns the submitted one-time code without any trimming or normalization.
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Returns the submitted CSRF secret exactly as submitted.
    pub fn csrf(&self) -> Option<&str> {
        self.csrf.as_deref()
    }
}
