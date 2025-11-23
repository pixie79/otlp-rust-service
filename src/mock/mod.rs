//! Mock service module
//!
//! Provides mock OTLP service for testing both gRPC interface and public API methods.

pub mod service;

pub use service::MockOtlpService;

