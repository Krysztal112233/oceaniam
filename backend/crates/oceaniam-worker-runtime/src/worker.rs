//! Business handlers and their optional cron producer registrations.

use std::{fmt, sync::Arc};

use async_trait::async_trait;

// clippy 1.99 double_must_use fires on async_trait macro expansions.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait Worker<Ctx: Send + Sync>: Send + Sync {
    type Error: fmt::Display;

    fn name(&self) -> &'static str;

    /// Handles a trigger. Delivery is at least once, so implementations must be idempotent.
    async fn run(&self, context: &Ctx) -> Result<(), Self::Error>;
}

/// A handler is always available to the queue consumer; cron only adds a message producer.
pub struct WorkerRegistration<Ctx: Send + Sync, E> {
    pub worker: Arc<dyn Worker<Ctx, Error = E>>,
    /// UTC cron expression, or `None` for a message-only worker.
    pub cron: Option<&'static str>,
}
