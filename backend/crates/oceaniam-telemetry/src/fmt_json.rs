use std::fmt;

use opentelemetry::trace::TraceContextExt;
use serde_json::{Map, Value, json};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
use tracing_subscriber::fmt::time::{FormatTime, SystemTime};
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

#[derive(Debug, Clone)]
pub struct RecordedTraceIds {
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TraceIdLayer;

impl<S> Layer<S> for TraceIdLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, _attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        record_trace_ids(id, &ctx);
    }

    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        record_trace_ids(id, &ctx);
    }
}

fn record_trace_ids<S>(id: &Id, ctx: &Context<'_, S>)
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let Some(span) = ctx.span(id) else {
        return;
    };
    if span.extensions().get::<RecordedTraceIds>().is_some() {
        return;
    }
    let Some(ids) = otel_ids_for_span_id(id) else {
        return;
    };
    span.extensions_mut().insert(ids);
}

#[derive(Debug, Default)]
pub(crate) struct JsonWithTraceIds {
    timer: SystemTime,
}

impl<S, N> FormatEvent<S, N> for JsonWithTraceIds
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut timestamp = String::new();
        self.timer.format_time(&mut Writer::new(&mut timestamp))?;

        let metadata = event.metadata();
        let mut fields = Map::new();
        event.record(&mut JsonFieldVisitor(&mut fields));

        let mut root = Map::new();
        root.insert("timestamp".into(), Value::String(timestamp));
        root.insert("level".into(), Value::String(metadata.level().to_string()));
        root.insert("fields".into(), Value::Object(fields));
        root.insert("target".into(), Value::String(metadata.target().into()));

        let current_span = event
            .parent()
            .and_then(|id| ctx.span(id))
            .or_else(|| ctx.lookup_current());
        if let Some(span) = current_span.as_ref() {
            if let Some(ids) = lookup_recorded_ids(span) {
                root.insert("trace_id".into(), Value::String(ids.trace_id));
                root.insert("span_id".into(), Value::String(ids.span_id));
            }
            root.insert("span".into(), formatted_span::<S, N>(span));
            root.insert(
                "spans".into(),
                Value::Array(
                    span.scope()
                        .from_root()
                        .map(|span| formatted_span::<S, N>(&span))
                        .collect(),
                ),
            );
        }

        writeln!(writer, "{}", Value::Object(root)).map_err(|_| fmt::Error)
    }
}

fn formatted_span<S, N>(span: &tracing_subscriber::registry::SpanRef<'_, S>) -> Value
where
    S: for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    let mut fields = span
        .extensions()
        .get::<FormattedFields<N>>()
        .map(
            |formatted| match serde_json::from_str::<Value>(&formatted.fields) {
                Ok(Value::Object(fields)) => fields,
                Ok(value) => Map::from_iter([
                    ("field".into(), value),
                    (
                        "field_error".into(),
                        Value::String("span fields were not a JSON object".into()),
                    ),
                ]),
                Err(error) => {
                    Map::from_iter([("field_error".into(), Value::String(error.to_string()))])
                }
            },
        )
        .unwrap_or_default();
    fields.insert("name".into(), Value::String(span.name().into()));
    Value::Object(fields)
}

fn lookup_recorded_ids<S>(
    span: &tracing_subscriber::registry::SpanRef<'_, S>,
) -> Option<RecordedTraceIds>
where
    S: for<'a> LookupSpan<'a>,
{
    for span in span.scope() {
        if let Some(ids) = span.extensions().get::<RecordedTraceIds>().cloned() {
            return Some(ids);
        }
        if let Some(ids) = otel_ids_for_span_id(&span.id()) {
            return Some(ids);
        }
    }
    None
}

pub fn current_otel_ids() -> Option<(String, String)> {
    let span = tracing::Span::current();
    let id = span.id()?;
    let ids = otel_ids_for_span_id(&id)?;
    Some((ids.trace_id, ids.span_id))
}

fn otel_ids_for_span_id(span_id: &Id) -> Option<RecordedTraceIds> {
    let context = tracing::dispatcher::get_default(|dispatch| {
        tracing_opentelemetry::get_otel_context(span_id, dispatch)
    })?;
    let span = context.span();
    let span_context = span.span_context();
    if !span_context.is_valid() {
        return None;
    }
    Some(RecordedTraceIds {
        trace_id: span_context.trace_id().to_string(),
        span_id: span_context.span_id().to_string(),
    })
}

struct JsonFieldVisitor<'a>(&'a mut Map<String, Value>);

impl Visit for JsonFieldVisitor<'_> {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().into(), json!(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().into(), json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().into(), json!(value));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.0
            .insert(field.name().into(), Value::String(value.to_string()));
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.0
            .insert(field.name().into(), Value::String(value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().into(), Value::Bool(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0
            .insert(field.name().into(), Value::String(value.to_owned()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0
            .insert(field.name().into(), Value::String(format!("{value:?}")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use tracing::Instrument;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    #[derive(Clone, Default)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);

    impl Write for Buffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("buffer lock").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Buffer {
        type Writer = Buffer;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn last_json_line(buf: &Buffer) -> Value {
        let bytes = buf.0.lock().expect("buffer lock").clone();
        let text = String::from_utf8(bytes).expect("utf8 log buffer");
        let line = text
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .expect("expected at least one log line");
        serde_json::from_str(line)
            .unwrap_or_else(|error| panic!("invalid json log line {line:?}: {error}"))
    }

    // NOTE: AI-generated test
    #[test]
    fn omits_trace_ids_without_otel_layer() {
        let buffer = Buffer::default();
        let _guard = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(JsonWithTraceIds::default())
                    .with_writer(buffer.clone())
                    .with_ansi(false),
            )
            .with(TraceIdLayer)
            .set_default();

        let span = tracing::info_span!("local.only");
        let _entered = span.entered();
        tracing::info!("no otel");

        let line = last_json_line(&buffer);
        assert_eq!(line["fields"]["message"], "no otel");
        assert!(line.get("trace_id").is_none());
        assert!(line.get("span_id").is_none());
        assert_eq!(line["span"]["name"], "local.only");
    }

    // NOTE: AI-generated test
    #[test]
    fn includes_recorded_span_fields_and_scope() {
        let buffer = Buffer::default();
        let _guard = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .fmt_fields(tracing_subscriber::fmt::format::JsonFields::new())
                    .event_format(JsonWithTraceIds::default())
                    .with_writer(buffer.clone())
                    .with_ansi(false),
            )
            .set_default();

        let root = tracing::info_span!("request", request_id = 7u64);
        let child =
            tracing::info_span!(parent: &root, "endpoint", tenant_id = tracing::field::Empty);
        child.record("tenant_id", "tenant-a");
        let _entered = child.entered();
        tracing::info!("nested event");

        let line = last_json_line(&buffer);
        assert_eq!(line["span"]["name"], "endpoint");
        assert_eq!(line["span"]["tenant_id"], "tenant-a");
        assert_eq!(line["spans"][0]["name"], "request");
        assert_eq!(line["spans"][0]["request_id"], 7);
        assert_eq!(line["spans"][1]["name"], "endpoint");
        assert_eq!(line["spans"][1]["tenant_id"], "tenant-a");
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn includes_trace_ids_inside_otel_span() {
        let provider = SdkTracerProvider::builder().build();
        let tracer = provider.tracer("oceaniam-telemetry-test");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        let buffer = Buffer::default();
        let _guard = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(JsonWithTraceIds::default())
                    .with_writer(buffer.clone())
                    .with_ansi(false),
            )
            .with(otel_layer)
            .with(TraceIdLayer)
            .set_default();

        async { tracing::info!(user_id = 42u64, "correlated") }
            .instrument(tracing::info_span!("http.request"))
            .await;

        let line = last_json_line(&buffer);
        assert_eq!(line["fields"]["message"], "correlated");
        assert_eq!(line["fields"]["user_id"], 42);
        assert_eq!(line["span"]["name"], "http.request");
        let trace_id = line["trace_id"]
            .as_str()
            .unwrap_or_else(|| panic!("missing trace_id in {line}"));
        let span_id = line["span_id"]
            .as_str()
            .unwrap_or_else(|| panic!("missing span_id in {line}"));
        assert_eq!(trace_id.len(), 32);
        assert_eq!(span_id.len(), 16);
        assert_ne!(trace_id, "00000000000000000000000000000000");
        assert_ne!(span_id, "0000000000000000");

        let _ = provider.shutdown();
    }
}
