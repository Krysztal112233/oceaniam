use oceaniam_common::config::TelemetryConfig;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, SpanExporter, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use snafu::ResultExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::error::{ConfigSnafu, EnvFilterSnafu, ExporterSnafu};
use crate::{Error, fmt_json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessKind {
    Server,
    Worker,
}

impl ProcessKind {
    pub fn service_name(self, base: &str) -> String {
        match self {
            Self::Server => base.to_owned(),
            Self::Worker => format!("{base}-worker"),
        }
    }
}

#[must_use = "dropping TelemetryGuard shuts down OTLP exporters; bind it in main"]
pub struct TelemetryGuard {
    logger_provider: Option<SdkLoggerProvider>,
    tracer_provider: Option<SdkTracerProvider>,
}

impl TelemetryGuard {
    pub fn shutdown(mut self) {
        self.shutdown_inner();
    }

    fn shutdown_inner(&mut self) {
        if let Some(provider) = self.tracer_provider.take()
            && let Err(err) = provider.shutdown()
        {
            eprintln!("failed to shut down OTLP tracer provider: {err}");
        }
        if let Some(provider) = self.logger_provider.take()
            && let Err(err) = provider.shutdown()
        {
            eprintln!("failed to shut down OTLP logger provider: {err}");
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        self.shutdown_inner();
    }
}

pub fn init(
    config: &TelemetryConfig,
    kind: ProcessKind,
    default_filter: &str,
) -> Result<TelemetryGuard, Error> {
    use tracing_subscriber::Layer;

    validate_trace_sample_ratio(config.trace_sample_ratio)?;

    let service_name = kind.service_name(&config.service_name);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .fmt_fields(tracing_subscriber::fmt::format::JsonFields::new())
        .event_format(fmt_json::JsonWithTraceIds::default())
        .with_ansi(false)
        .with_filter(env_filter(default_filter));

    if !config.enabled {
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(fmt_json::TraceIdLayer)
            .init();

        return Ok(TelemetryGuard {
            logger_provider: None,
            tracer_provider: None,
        });
    }

    let logs_endpoint = config.otlp_endpoint.as_ref();
    let traces_endpoint = config.otlp_traces_endpoint.as_ref();
    if logs_endpoint.is_none() && traces_endpoint.is_none() {
        return Err(ConfigSnafu {
            msg: "telemetry.enabled is true but neither telemetry.otlp_endpoint nor telemetry.otlp_traces_endpoint is set",
        }
        .build());
    }

    let resource = build_resource(&service_name, kind);
    let logger_provider = logs_endpoint
        .map(|endpoint| build_logger_provider(endpoint.as_str(), &resource, config))
        .transpose()?;
    let tracer_provider = traces_endpoint
        .map(|endpoint| build_tracer_provider(endpoint.as_str(), &resource, config))
        .transpose()?;

    if let Some(provider) = &tracer_provider {
        global::set_tracer_provider(provider.clone());
    }

    let logs_layer = logger_provider
        .as_ref()
        .map(|provider| {
            Ok::<_, Error>(
                OpenTelemetryTracingBridge::new(provider)
                    .with_filter(otel_export_filter(default_filter)?),
            )
        })
        .transpose()?;
    let traces_layer = tracer_provider
        .as_ref()
        .map(|provider| {
            let tracer = provider.tracer(service_name.clone());
            Ok::<_, Error>(
                tracing_opentelemetry::layer()
                    .with_tracer(tracer)
                    .with_filter(otel_export_filter(default_filter)?),
            )
        })
        .transpose()?;

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(logs_layer)
        .with(traces_layer)
        .with(fmt_json::TraceIdLayer)
        .init();

    tracing::info!(
        %service_name,
        otlp_logs_endpoint = logs_endpoint.map(|url| url.as_str()),
        otlp_traces_endpoint = traces_endpoint.map(|url| url.as_str()),
        "OTLP telemetry enabled"
    );

    Ok(TelemetryGuard {
        logger_provider,
        tracer_provider,
    })
}

fn build_resource(service_name: &str, kind: ProcessKind) -> Resource {
    Resource::builder()
        .with_service_name(service_name.to_owned())
        .with_attributes([
            KeyValue::new(
                "service.version",
                option_env!("CARGO_PKG_VERSION").unwrap_or("0.0.0"),
            ),
            KeyValue::new(
                "process.kind",
                match kind {
                    ProcessKind::Server => "server",
                    ProcessKind::Worker => "worker",
                },
            ),
        ])
        .build()
}

fn build_logger_provider(
    endpoint: &str,
    resource: &Resource,
    config: &TelemetryConfig,
) -> Result<SdkLoggerProvider, Error> {
    let mut exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(endpoint);
    if !config.otlp_headers.is_empty() {
        exporter = exporter.with_headers(config.otlp_headers.clone());
    }
    let exporter = exporter.build().map_err(|error| {
        ExporterSnafu {
            msg: error.to_string(),
        }
        .build()
    })?;

    Ok(SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(exporter)
        .build())
}

fn build_tracer_provider(
    endpoint: &str,
    resource: &Resource,
    config: &TelemetryConfig,
) -> Result<SdkTracerProvider, Error> {
    let mut exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(endpoint);
    if !config.otlp_headers.is_empty() {
        exporter = exporter.with_headers(config.otlp_headers.clone());
    }
    let exporter = exporter.build().map_err(|error| {
        ExporterSnafu {
            msg: error.to_string(),
        }
        .build()
    })?;

    Ok(SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_sampler(trace_sampler(config.trace_sample_ratio))
        .with_batch_exporter(exporter)
        .build())
}

fn validate_trace_sample_ratio(ratio: f64) -> Result<(), Error> {
    if ratio.is_finite() && (0.0..=1.0).contains(&ratio) {
        Ok(())
    } else {
        Err(ConfigSnafu {
            msg: format!(
                "telemetry.trace_sample_ratio must be finite and within 0.0..=1.0, got {ratio}"
            ),
        }
        .build())
    }
}

fn trace_sampler(ratio: f64) -> Sampler {
    Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
}

fn env_filter(default_filter: &str) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter))
}

fn otel_export_filter(default_filter: &str) -> Result<EnvFilter, Error> {
    Ok(env_filter(default_filter)
        .add_directive("hyper=off".parse().context(EnvFilterSnafu)?)
        .add_directive("hyper_util=off".parse().context(EnvFilterSnafu)?)
        .add_directive("h2=off".parse().context(EnvFilterSnafu)?)
        .add_directive("reqwest=off".parse().context(EnvFilterSnafu)?)
        .add_directive("opentelemetry=off".parse().context(EnvFilterSnafu)?)
        .add_directive("opentelemetry_sdk=off".parse().context(EnvFilterSnafu)?)
        .add_directive("opentelemetry_otlp=off".parse().context(EnvFilterSnafu)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn worker_appends_service_name_suffix() {
        assert_eq!(ProcessKind::Server.service_name("oceaniam"), "oceaniam");
        assert_eq!(
            ProcessKind::Worker.service_name("oceaniam"),
            "oceaniam-worker"
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn enabled_without_endpoints_is_rejected() {
        let config = TelemetryConfig {
            enabled: true,
            ..TelemetryConfig::default()
        };

        let error = init(&config, ProcessKind::Server, "info")
            .err()
            .expect("missing endpoints must be rejected");
        assert!(
            error
                .to_string()
                .contains("neither telemetry.otlp_endpoint")
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn trace_sample_ratio_validation_accepts_bounds_and_rejects_invalid_values() {
        assert_eq!(TelemetryConfig::default().trace_sample_ratio, 1.0);
        assert!(validate_trace_sample_ratio(0.0).is_ok());
        assert!(validate_trace_sample_ratio(1.0).is_ok());
        assert!(validate_trace_sample_ratio(f64::NAN).is_err());
        assert!(validate_trace_sample_ratio(f64::INFINITY).is_err());
        assert!(validate_trace_sample_ratio(-0.1).is_err());
        assert!(validate_trace_sample_ratio(1.1).is_err());
    }

    // NOTE: AI-generated test
    #[test]
    fn trace_sampler_respects_zero_and_full_ratios() {
        use opentelemetry::trace::{Tracer as _, TracerProvider as _};
        use opentelemetry_sdk::trace::{InMemorySpanExporter, SimpleSpanProcessor};

        for (ratio, expected) in [(0.0, 0), (1.0, 1)] {
            let exporter = InMemorySpanExporter::default();
            let provider = SdkTracerProvider::builder()
                .with_sampler(trace_sampler(ratio))
                .with_span_processor(SimpleSpanProcessor::new(exporter.clone()))
                .build();
            let tracer = provider.tracer("sampling-test");
            drop(tracer.start("root"));
            provider.force_flush().expect("flush spans");
            assert_eq!(
                exporter.get_finished_spans().expect("finished spans").len(),
                expected
            );
            provider.shutdown().expect("shutdown provider");
        }
    }
}
