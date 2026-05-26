use std::fmt;

use async_trait::async_trait;

#[async_trait]
pub trait Worker<Ctx: Send + Sync>: Send + Sync {
    type Error: fmt::Display;

    fn name(&self) -> &'static str;
    fn cron(&self) -> &'static str;

    async fn run(&self, context: &Ctx) -> Result<(), Self::Error>;
}
