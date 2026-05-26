use std::{collections::HashMap, fmt, sync::Arc};

use async_trait::async_trait;
use oceaniam_worker_runtime::{Worker, WorkerRuntime, WorkerRuntimeError};

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

    fn cron(&self) -> &'static str {
        "*/1 * * * * *"
    }

    async fn run(&self, context: &Context) -> Result<(), Self::Error> {
        println!("hello {} from worker `{}`", context.label, self.name());
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let ctx = Context {
        label: "oceaniam".into(),
    };

    let mut workers: HashMap<String, Arc<dyn Worker<Context, Error = Error>>> = HashMap::new();
    workers.insert("hello".into(), Arc::new(SimpleHelloWorker));

    let runtime = WorkerRuntime::new(ctx, workers);
    let controller = runtime.start().expect("failed to start runtime");

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    controller.shutdown().await.expect("failed to shutdown");
    println!("runtime shut down cleanly");
}
