//! HTTP request spans for internal tracing (no inbound context propagation).

use std::time::Duration;

use http::{Request, Response};
use opentelemetry::trace::{Status, TraceContextExt};
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{
    DefaultOnBodyChunk, DefaultOnEos, DefaultOnFailure, DefaultOnRequest, TraceLayer,
};
use tracing::field::Empty;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// `TraceLayer` that creates a fresh server span for each inbound request.
/// Inbound trace context headers are intentionally ignored.
pub type HttpTraceLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    OtelMakeSpan,
    DefaultOnRequest,
    OtelOnResponse,
    DefaultOnBodyChunk,
    DefaultOnEos,
    DefaultOnFailure,
>;

pub fn trace_layer() -> HttpTraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(OtelMakeSpan)
        .on_response(OtelOnResponse)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OtelMakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for OtelMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> tracing::Span {
        let method = request.method();
        let path = request.uri().path();
        let span_name = format!("HTTP {method}");

        let span = tracing::info_span!(
            "http.request",
            otel.name = %span_name,
            otel.kind = "server",
            "http.request.method" = %method,
            "http.route" = Empty,
            "url.path" = %path,
            "url.query" = Empty,
            "http.response.status_code" = Empty,
            "http.server.request.duration_ms" = Empty,
            "user_agent.original" = Empty,
            "enduser.id" = Empty,
            "user.is_admin" = Empty,
        );

        if !is_oidc_namespace(path)
            && let Some(query) = request.uri().query()
        {
            span.record("url.query", query);
        }
        if let Some(user_agent) = request
            .headers()
            .get(http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
        {
            span.record("user_agent.original", user_agent);
        }

        span
    }
}

fn is_oidc_namespace(path: &str) -> bool {
    path == "/oidc" || path.starts_with("/oidc/")
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OtelOnResponse;

impl<B> tower_http::trace::OnResponse<B> for OtelOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &tracing::Span) {
        let status = response.status().as_u16();
        span.record("http.response.status_code", status);
        span.record(
            "http.server.request.duration_ms",
            latency.as_millis() as u64,
        );

        if response.status().is_server_error() {
            span.set_status(Status::error(format!("HTTP {status}")));
        }
    }
}

pub fn record_http_route(method: &http::Method, route: &str) {
    let span = tracing::Span::current();
    let span_name = format!("{method} {route}");
    span.record("http.route", route);
    span.record("otel.name", tracing::field::display(&span_name));
    span.context().span().update_name(span_name);
}

pub fn record_request_user(user_id: impl std::fmt::Display, is_admin: bool) {
    let span = tracing::Span::current();
    span.record("enduser.id", tracing::field::display(user_id));
    span.record("user.is_admin", is_admin);
}

#[cfg(test)]
mod tests {
    use super::*;

    use opentelemetry::{Value, trace::TracerProvider as _};
    use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider};
    use tower_http::trace::MakeSpan as _;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // NOTE: AI-generated test
    #[test]
    fn oidc_namespace_redacts_query_while_ordinary_routes_retain_it() {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("oceaniam-telemetry-query-test");
        let _guard = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .set_default();

        let mut make_span = OtelMakeSpan;
        let oidc_request = Request::builder()
            .uri("/oidc/tenant/authorize?state=oidc-query-sentinel")
            .body(())
            .expect("OIDC request");
        let ordinary_request = Request::builder()
            .uri("/tenants?marker=ordinary-query-sentinel")
            .body(())
            .expect("ordinary request");
        drop(make_span.make_span(&oidc_request));
        drop(make_span.make_span(&ordinary_request));

        provider.force_flush().expect("flush spans");
        let spans = exporter.get_finished_spans().expect("finished spans");
        assert_eq!(spans.len(), 2);

        let attribute = |path: &str, name: &str| {
            spans
                .iter()
                .find(|span| {
                    span.attributes.iter().any(|attribute| {
                        attribute.key.as_str() == "url.path"
                            && matches!(&attribute.value, Value::String(value) if value.as_str() == path)
                    })
                })
                .and_then(|span| {
                    span.attributes
                        .iter()
                        .find(|attribute| attribute.key.as_str() == name)
                })
                .map(|attribute| attribute.value.clone())
        };
        assert!(
            attribute("/oidc/tenant/authorize", "url.query").is_none(),
            "OIDC requests must never export their query"
        );
        assert!(matches!(
            attribute("/tenants", "url.query"),
            Some(Value::String(value)) if value.as_str() == "marker=ordinary-query-sentinel"
        ));

        provider.shutdown().expect("shutdown provider");
    }

    // NOTE: AI-generated test
    #[test]
    fn matched_route_updates_exported_span_name() {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("oceaniam-telemetry-http-test");
        let _guard = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .set_default();

        {
            let span = tracing::info_span!(
                "http.request",
                otel.name = "HTTP GET",
                otel.kind = "server",
                "http.route" = tracing::field::Empty,
            );
            let _entered = span.entered();
            record_http_route(&http::Method::GET, "/tenants/{tenant_id}");
        }

        provider.force_flush().expect("flush spans");
        let spans = exporter.get_finished_spans().expect("finished spans");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "GET /tenants/{tenant_id}");

        provider.shutdown().expect("shutdown provider");
    }
}
