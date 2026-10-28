use std::fmt;

use async_trait::async_trait;

// clippy 1.99 double_must_use fires on async_trait macro expansions.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait Worker<Ctx: Send + Sync>: Send + Sync {
    type Error: fmt::Display;

    fn name(&self) -> &'static str;
    fn cron(&self) -> &'static str;

    async fn run(&self, context: &Ctx) -> Result<(), Self::Error>;
}
