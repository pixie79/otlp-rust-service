//! Performance benchmark for circuit breaker lock acquisition frequency
//!
//! Measures lock acquisition frequency before and after optimization to verify
//! that state updates are batched into fewer lock operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use otlp_arrow_library::config::{ForwardingConfig, ForwardingProtocol};
use otlp_arrow_library::otlp::forwarder::OtlpForwarder;
use otlp_arrow_library::error::OtlpError;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

/// Helper function to create a test span
fn create_test_span(name: &str) -> SpanData {
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    
    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );
    
    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Server,
        name: std::borrow::Cow::Owned(name.to_string()),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now() + Duration::from_secs(1),
        attributes: vec![].into_iter().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("bench")
            .with_version("1.0.0")
            .build(),
    }
}

fn bench_circuit_breaker_lock_acquisition(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Create a mock server that always succeeds (to avoid opening circuit breaker)
    let mock_server = rt.block_on(async {
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use wiremock::matchers::{method, path};
        
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/traces"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        server
    });
    
    let forwarding_config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some(format!("http://{}", mock_server.address())),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let forwarder = Arc::new(rt.block_on(async {
        OtlpForwarder::new(forwarding_config).unwrap()
    }));
    
    let spans = vec![create_test_span("bench-span")];
    
    // Benchmark single forward operation (measures lock acquisition overhead)
    c.bench_function("circuit_breaker_single_forward", |b| {
        let forwarder_clone = forwarder.clone();
        let spans_clone = spans.clone();
        b.to_async(&rt).iter(|| {
            let forwarder = forwarder_clone.clone();
            let spans = spans_clone.clone();
            async move {
                black_box(forwarder.forward_traces(spans).await)
            }
        });
    });
    
    // Benchmark concurrent forwards (measures lock contention)
    let mut group = c.benchmark_group("circuit_breaker_concurrent");
    for concurrency in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &concurrency| {
                let forwarder_clone = forwarder.clone();
                let spans_clone = spans.clone();
                b.to_async(&rt).iter(|| {
                    let forwarder = forwarder_clone.clone();
                    let spans = spans_clone.clone();
                    async move {
                        let mut join_set = JoinSet::new();
                        for _ in 0..concurrency {
                            let forwarder = forwarder.clone();
                            let spans = spans.clone();
                            join_set.spawn(async move {
                                forwarder.forward_traces(spans).await
                            });
                        }
                        
                        while let Some(result) = join_set.join_next().await {
                            black_box(result);
                        }
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_circuit_breaker_state_transitions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Benchmark state transition overhead (Closed -> Open -> HalfOpen -> Closed)
    c.bench_function("circuit_breaker_state_transition", |b| {
        let rt_clone = rt.clone();
        b.to_async(&rt).iter(|| {
            async move {
                // Create a new forwarder for each iteration to measure state transition overhead
                use wiremock::{Mock, MockServer, ResponseTemplate};
                use wiremock::matchers::{method, path};
                
                let server = MockServer::start().await;
                // First request succeeds, then failures to trigger state transitions
                let mut mock = Mock::given(method("POST")).and(path("/v1/traces"));
                mock = mock.respond_with(ResponseTemplate::new(200)); // First success
                for _ in 0..6 {
                    mock = mock.respond_with(ResponseTemplate::new(500)); // Failures to open
                }
                mock.mount(&server).await;
                
                let config = ForwardingConfig {
                    enabled: true,
                    endpoint_url: Some(format!("http://{}", server.address())),
                    protocol: ForwardingProtocol::Protobuf,
                    authentication: None,
                };
                
                let forwarder = OtlpForwarder::new(config).unwrap();
                let spans = vec![create_test_span("transition-test")];
                
                // Trigger state transitions
                for _ in 0..7 {
                    let _ = forwarder.forward_traces(spans.clone()).await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                
                black_box(())
            }
        });
    });
}

criterion_group!(
    benches,
    bench_circuit_breaker_lock_acquisition,
    bench_circuit_breaker_state_transitions
);
criterion_main!(benches);
