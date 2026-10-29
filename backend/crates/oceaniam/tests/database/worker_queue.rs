//! One end-to-end guard for the queue boundary, using a disposable PostgreSQL database.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use oceaniam_database::helper::{pgmq, worker_jobs};
use oceaniam_worker::error::Error;
use oceaniam_worker_runtime::{Worker, WorkerRegistration, WorkerRuntime, WorkerRuntimeError};
use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait,
};
use tokio::sync::Notify;
use uuid::Uuid;

struct TestDatabase {
    base_dsn: String,
    name: String,
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let base_dsn = self.base_dsn.clone();
        let name = self.name.clone();
        // A separate runtime also cleans up during test unwinding. FORCE closes any remaining
        // consumer connections, but only inside this test's disposable database.
        let cleanup = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("create database cleanup runtime");
            runtime.block_on(async move {
                let base = Database::connect(&base_dsn).await?;
                base.execute_raw(Statement::from_string(
                    DatabaseBackend::Postgres,
                    format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"),
                ))
                .await?;
                base.close().await
            })
        })
        .join();
        match cleanup {
            Ok(Ok(())) => {}
            failure if std::thread::panicking() => {
                eprintln!(
                    "failed to drop queue test database {}: {failure:?}",
                    self.name
                );
            }
            failure => panic!(
                "failed to drop queue test database {}: {failure:?}",
                self.name
            ),
        }
    }
}

async fn isolated_database() -> (TestDatabase, DatabaseConnection) {
    dotenvy::dotenv().ok();
    let base_dsn = std::env::var("OCEANIAM_TEST_DATABASE_DSN")
        .or_else(|_| std::env::var("OCEANIAM_DATABASE__DSN"))
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/postgres".to_owned());
    let name = format!("test_worker_queue_{}", Uuid::now_v7().simple());
    let mut dsn = url::Url::parse(&base_dsn).expect("parse PostgreSQL test DSN");
    dsn.set_path(&name);

    let base = Database::connect(&base_dsn)
        .await
        .expect("connect for queue test database creation");
    base.execute_raw(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("CREATE DATABASE {name}"),
    ))
    .await
    .expect("create isolated queue test database (test role requires CREATEDB permission)");
    // Install the guard before any subsequent setup can fail.
    let guard = TestDatabase { base_dsn, name };
    base.close().await.expect("close database setup connection");

    let database = Database::connect(dsn.as_str())
        .await
        .expect("connect to isolated queue test database");
    database
        .execute_unprepared(
            "CREATE EXTENSION IF NOT EXISTS pgmq; SELECT pgmq.create('worker_jobs')",
        )
        .await
        .expect("initialize PGMQ in isolated queue test database");
    (guard, database)
}

#[derive(Default)]
struct Probe {
    attempts: AtomicUsize,
    started: Notify,
    release: Notify,
}

struct QueueProbe;

#[async_trait]
impl Worker<Arc<Probe>> for QueueProbe {
    type Error = Error;

    fn name(&self) -> &'static str {
        "queue_probe"
    }

    async fn run(&self, probe: &Arc<Probe>) -> Result<(), Error> {
        let attempt = probe.attempts.fetch_add(1, Ordering::SeqCst);
        probe.started.notify_one();
        if attempt == 0 {
            return Err(Error::internal("retry probe".to_owned()));
        }
        probe.release.notified().await;
        Ok(())
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn queue_fences_retries_and_drains_without_cron() {
    let (_test_database, database) = isolated_database().await;
    // A redelivered message invalidates the previous claim's renew/ack/archive authority.
    worker_jobs::enqueue("queue_probe", 0, &database)
        .await
        .unwrap();
    let first = worker_jobs::read(0, 1, &database).await.unwrap().remove(0);
    let current = worker_jobs::read(30, 1, &database).await.unwrap().remove(0);
    assert_eq!(first.msg_id, current.msg_id);
    assert_eq!(current.read_ct, first.read_ct + 1);
    assert!(!worker_jobs::renew(&first, 30, &database).await.unwrap());
    assert!(!worker_jobs::ack(&first, &database).await.unwrap());
    assert!(!worker_jobs::archive(&first, &database).await.unwrap());
    assert!(worker_jobs::renew(&current, 30, &database).await.unwrap());
    assert!(worker_jobs::archive(&current, &database).await.unwrap());

    // A trigger participating in a rolled-back business transaction must not escape it.
    let transaction = database.begin().await.unwrap();
    worker_jobs::enqueue("queue_probe", 0, &transaction)
        .await
        .unwrap();
    transaction.rollback().await.unwrap();
    assert!(
        worker_jobs::read(30, 1, &database)
            .await
            .unwrap()
            .is_empty()
    );

    let malformed = pgmq::send(
        worker_jobs::WORKER_QUEUE,
        serde_json::json!({}),
        0,
        &database,
    )
    .await
    .unwrap();
    let trigger = worker_jobs::enqueue("queue_probe", 0, &database)
        .await
        .unwrap();
    let probe = Arc::new(Probe::default());
    let registrations = HashMap::from([(
        "queue_probe".to_owned(),
        WorkerRegistration {
            worker: Arc::new(QueueProbe) as Arc<dyn Worker<Arc<Probe>, Error = Error>>,
            cron: None,
        },
    )]);
    let controller = WorkerRuntime::new(probe.clone(), registrations, database.clone())
        .start()
        .expect("start message-only runtime");
    tokio::time::timeout(Duration::from_secs(5), probe.started.notified())
        .await
        .expect("manual trigger runs without a cron");

    // Simulate lease expiry without waiting two minutes; the failed attempt left the job queued.
    database
        .query_one_raw(Statement::from_sql_and_values(
            database.get_database_backend(),
            "SELECT pgmq.set_vt('worker_jobs', $1, 0)",
            [trigger.into()],
        ))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), probe.started.notified())
        .await
        .expect("failed trigger is redelivered");
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 2);

    let mut shutdown = tokio::spawn(controller.shutdown());
    assert!(
        tokio::time::timeout(Duration::from_millis(50), &mut shutdown)
            .await
            .is_err(),
        "shutdown must wait for the blocked handler"
    );
    probe.release.notify_one();
    tokio::time::timeout(Duration::from_secs(5), shutdown)
        .await
        .expect("shutdown completes")
        .expect("shutdown task")
        .expect("runtime shutdown");
    assert!(
        worker_jobs::read(30, 1, &database)
            .await
            .unwrap()
            .is_empty(),
        "successful retry must ack its trigger"
    );
    let archived = database
        .query_one_raw(Statement::from_sql_and_values(
            database.get_database_backend(),
            "SELECT msg_id FROM pgmq.a_worker_jobs WHERE msg_id = $1",
            [malformed.into()],
        ))
        .await
        .unwrap();
    assert!(
        archived.is_some(),
        "malformed messages must be archived, not retried forever"
    );
}
