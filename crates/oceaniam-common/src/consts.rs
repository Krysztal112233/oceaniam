use jsonwebtoken::Algorithm;
use uuid::{Uuid, uuid};

/// [Uuid] allocated as system if self
pub const SYSTEM_APPLICATION_UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

/// [Uuid] allocated as system if self
pub const SYSTEM_TENANT_UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

/// TODO: make this field configurable
pub const SYSTEM_KEY_ALO: Algorithm = Algorithm::PS512;
