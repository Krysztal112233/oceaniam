use std::time::Duration;

use jsonwebtoken::Algorithm;
use uuid::{Uuid, uuid};

/// [Uuid] allocated as system if self
pub const SYSTEM_APPLICATION_UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

/// [Uuid] allocated as system if self
pub const SYSTEM_TENANT_UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

/// TODO: make this field configurable
pub const SYSTEM_KEY_ALO: Algorithm = Algorithm::PS512;

pub const DEFAULT_KEY_EXPIRES_AFTER: Duration = Duration::from_hours(24 * 30);

pub const DEFAULT_KEY_RETIED_AFTER: Duration = Duration::from_hours(24 * 30);

pub const USER_LOGIN_FAILED_MSG: &str = "user not found or password invalid";
