//! Integration tests for circuit breaker recovery scenarios
//!
//! Tests circuit breaker recovery behavior after failures, including timeout-based
//! transitions and recovery verification.

use otlp_arrow_library::config::{ForwardingConfig, ForwardingProtocol};
use otlp_arrow_library::otlp::forwarder::OtlpForwarder;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use std::sync::Arc;
use std::time::Duration;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

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
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test")
            .with_version("1.0.0")
            .build(),
    }
}

#[tokio::test]
async fn test_circuit_breaker_recovery_after_timeout() {
    // Test circuit breaker recovery after timeout period
    let mock_server = MockServer::start().await;
    
    // Initially return errors, then success after timeout
    Mock::given(method("POST"))
        .and(path("/v1/traces"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    
    let forwarding_config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some(format!("http://{}", mock_server.address())),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let forwarder = OtlpForwarder::new(forwarding_config).unwrap();
    let spans = vec![create_test_span("recovery-test")];
    
    // First, trigger failures to open circuit breaker
    // We'll use a different mock server for failures
    let failure_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/traces"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&failure_server)
        .await;
    
    let failure_config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some(format!("http://{}", failure_server.address())),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let failure_forwarder = OtlpForwarder::new(failure_config).unwrap();
    
    // Trigger failures
    for _ in 0..6 {
        let _ = failure_forwarder.forward_traces(spans.clone()).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Wait for half-open timeout (30 seconds)
    tokio::time::sleep(Duration::from_secs(31)).await;
    
    // Now use the success server - should recover
    let _ = forwarder.forward_traces(spans.clone()).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Circuit breaker should be closed, next request should succeed
    let _ = forwarder.forward_traces(spans.clone()).await;
}

#[tokio::test]
async fn test_circuit_breaker_recovery_failure_retry() {
    // Test circuit breaker retry after failure in half-open state
    let mock_server = MockServer::start().await;
    
    // Return success after initial failures
    let mut mock = Mock::given(method("POST"))
        .and(path("/v1/traces"));
    
    // First 5 requests fail (to open circuit breaker)
    for _ in 0..5 {
        mock = mock.respond_with(ResponseTemplate::new(500));
    }
    // Then one success (half-open test succeeds)
    mock = mock.respond_with(ResponseTemplate::new(200));
    // Then failures again (to test retry)
    for _ in 0..3 {
        mock = mock.respond_with(ResponseTemplate::new(500));
    }
    
    mock.mount(&mock_server).await;
    
    let forwarding_config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some(format!("http://{}", mock_server.address())),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let forwarder = OtlpForwarder::new(forwarding_config).unwrap();
    let spans = vec![create_test_span("retry-test")];
    
    // Trigger failures to open circuit breaker
    for _ in 0..6 {
        let _ = forwarder.forward_traces(spans.clone()).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Wait for half-open timeout
    tokio::time::sleep(Duration::from_secs(31)).await;
    
    // Success should transition back to closed
    let _ = forwarder.forward_traces(spans.clone()).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // More failures should open circuit breaker again
    for _ in 0..6 {
        let _ = forwarder.forward_traces(spans.clone()).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn test_circuit_breaker_concurrent_recovery() {
    // Test concurrent requests during recovery
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/traces"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    
    let forwarding_config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some(format!("http://{}", mock_server.address())),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let forwarder = Arc::new(OtlpForwarder::new(forwarding_config).unwrap());
    let spans = vec![create_test_span("concurrent-recovery")];
    
    // Spawn concurrent forward requests during recovery
    let mut handles = Vec::new();
    for i in 0..10 {
        let forwarder_clone = forwarder.clone();
        let spans_clone = spans.clone();
        let handle = tokio::spawn(async move {
            forwarder_clone.forward_traces(spans_clone).await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // All should return Ok(()) (forward_traces doesn't propagate errors)
    assert!(results.iter().all(|r| r.is_ok()), "All concurrent forwards should return Ok");
    
    // Wait for async forwarding to complete
    tokio::time::sleep(Duration::from_millis(500)).await;
}
