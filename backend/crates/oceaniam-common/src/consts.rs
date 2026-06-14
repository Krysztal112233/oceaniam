use std::sync::LazyLock;

use tokio::sync::Semaphore;
use uuid::{Uuid, uuid};

/// [Uuid] allocated as system if self
pub const SYSTEM_APPLICATION_UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

/// [Uuid] allocated as system if self
pub const SYSTEM_TENANT_UUID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

pub const DEFAULT_JWT_AUDIENCE: &str = "OceanIAM";

pub const DEFAULT_JWT_ISSUER: &str = "OceanIAM";

pub static MAX_CPU_BOUND_SEMAPHORE: LazyLock<Semaphore> =
    LazyLock::new(|| Semaphore::new(num_cpus::get()));

pub const USER_LOGIN_FAILED_MSG: &str = "user not found or password invalid";

/// The current Key Encryption Key (KEK) version. Used by envelope encryption
/// to tag encrypted blobs so future KEK rotation can identify the generation.
pub const KEK_VERSION_CURRENT: u32 = 1;
