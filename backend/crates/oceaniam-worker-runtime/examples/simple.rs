//! Run against a migrated disposable OceanIAM database using DATABASE_URL.
//! Do not share it with production consumers: this example registers only the hello task.
//! Set cron to None to expose the handler only to manually published triggers.

use std::{collections::HashMap, fmt, sync::Arc};

use async_trait::async_trait;
use oceaniam_worker_runtime::{Worker, WorkerRegistration, WorkerRuntime, WorkerRuntimeError};
use sea_orm::Database;

#[derive(Clone)]
struct Context {
    label: String,
}

#[derive(Debug)]
struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl WorkerRuntimeError for Error {
    fn internal(msg: String) -> Self {
        Error(msg)
    }
}

struct SimpleHelloWorker;

#[async_trait]
impl Worker<Context> for SimpleHelloWorker {
    type Error = Error;

    fn name(&self) -> &'static str {
        "hello"
    }

    async fn run(&self, context: &Context) -> Result<(), Self::Error> {
        println!("hello {} from worker `{}`", context.label, self.name());
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let queue_database =
        Database::connect(std::env::var("DATABASE_URL").expect("set DATABASE_URL"))
            .await
            .expect("connect database");
    let context = Context {
        label: "oceaniam".into(),
    };
    let workers = HashMap::from([(
        "hello".to_owned(),
        WorkerRegistration {
            worker: Arc::new(SimpleHelloWorker) as Arc<dyn Worker<Context, Error = Error>>,
            cron: Some("*/1 * * * * *"),
        },
    )]);
    let controller = WorkerRuntime::new(context, workers, queue_database)
        .start()
        .expect("start runtime");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    controller.shutdown().await.expect("shutdown runtime");
}
