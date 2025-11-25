//! Python bindings module
//!
//! Provides PyO3 bindings for the OTLP Arrow Library to enable Python integration.

pub mod adapters;
pub mod bindings;

pub use bindings::*;
