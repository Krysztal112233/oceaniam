use std::{sync::Arc, time::Duration};

use crossbeam_queue::SegQueue;
use oceaniam_audit::types::AuditPayload;
use oceaniam_database::{
    helper::{
        audits::{AuditsHelper, audit_model_to_active_model},
        db_clock_now,
    },
    model::{self, prelude::Audits, sea_orm_active_enums::AuditType},
};
use opentelemetry::trace::{SpanContext, TraceContextExt as _};
use sea_orm::DatabaseConnection;
use tokio::time::sleep;
use tracing::{Instrument, Span, field::Empty};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;
use uuid::Uuid;

use super::AuditWriter;

#[derive(Debug)]
struct QueuedAudit {
    audit_type: AuditType,
    payload: serde_json::Value,
    source: SpanContext,
}

#[derive(Debug, Clone)]
pub(super) struct DatabaseAuditWriter {
    queue: Arc<SegQueue<QueuedAudit>>,
    database: DatabaseConnection,
}

impl DatabaseAuditWriter {
    pub fn new(database: DatabaseConnection) -> DatabaseAuditWriter {
        let queue = Arc::new(SegQueue::new());

        let it = Self { queue, database };

        let worker = it.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(50)).await;
                drain(&worker.queue, &worker.database);
            }
        });

        it
    }
}

#[async_trait::async_trait]
impl AuditWriter for DatabaseAuditWriter {
    async fn write(&self, payload: AuditPayload) {
        let context = Span::current().context();
        let source = context.span().span_context().clone();
        self.queue.push(QueuedAudit {
            audit_type: payload.audit_type(),
            payload: serde_json::to_value(payload).unwrap(),
            source,
        });
    }
}

fn drain(queue: &SegQueue<QueuedAudit>, database: &DatabaseConnection) {
    let mut audits = Vec::new();
    let mut sources = Vec::new();
    while let Some(QueuedAudit {
        audit_type,
        payload,
        source,
    }) = queue.pop()
    {
        audits.push((audit_type, payload));
        if source.is_valid() {
            sources.push(source);
        }
    }

    if audits.is_empty() {
        return;
    }

    let span = audit_flush_span(audits.len(), sources);

    let db = database.clone();
    tokio::spawn(
        async move {
            // The rows are materialized at flush time with a single database-clock sample
            // (Rule 3): `write()` is synchronous and cannot consult the time authority, so the
            // queued payload carries no timestamp of its own.
            let result = async {
                let created_at = db_clock_now(&db).await?;
                let models = audits
                    .into_iter()
                    .map(|(audit_type, payload)| {
                        audit_model_to_active_model(model::audits::Model {
                            id: Uuid::now_v7(),
                            audit_type,
                            payload,
                            created_at,
                        })
                    })
                    .collect();
                Audits::insert_many_audits(models, &db).await
            }
            .await;
            if let Err(error) = result {
                let span = Span::current();
                span.record("otel.status_code", "ERROR");
                span.record("otel.status_description", "audit batch insert failed");
                tracing::error!(%error, "failed to flush audit batch");
            }
        }
        .instrument(span),
    );
}

fn audit_flush_span(batch_size: usize, sources: Vec<SpanContext>) -> tracing::Span {
    let valid_sources = sources
        .into_iter()
        .filter(SpanContext::is_valid)
        .collect::<Vec<_>>();
    let span = tracing::info_span!(
        parent: None,
        "audit.flush",
        otel.kind = "internal",
        otel.status_code = Empty,
        otel.status_description = Empty,
        audit.batch_size = batch_size,
        audit.link_count = valid_sources.len(),
    );
    for source in valid_sources {
        span.add_link(source);
    }
    span
}

#[cfg(test)]
mod tests {
    use super::*;

    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider};
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    // NOTE: AI-generated test
    #[test]
    fn audit_flush_links_only_valid_source_contexts() {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("audit-link-test");
        let _default = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .set_default();

        let source = tracing::info_span!("request.source");
        let source_context = source.context();
        let source_span_context = source_context.span().span_context().clone();
        drop(source);
        drop(audit_flush_span(
            2,
            vec![source_span_context, SpanContext::empty_context()],
        ));

        provider.force_flush().expect("flush spans");
        let spans = exporter.get_finished_spans().expect("finished spans");
        let flush = spans
            .iter()
            .find(|span| span.name == "audit.flush")
            .expect("audit flush span");
        assert_eq!(flush.links.len(), 1);

        provider.shutdown().expect("shutdown provider");
    }
}
