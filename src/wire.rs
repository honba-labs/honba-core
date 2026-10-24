//! Distributed observability and wire transport context.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Zero-dependency W3C trace context carrier.
///
/// Implements the W3C Trace Context Level 1 specification (`traceparent` header).
/// Enables end-to-end distributed tracing across microservices, asynchronous actor
/// hops, audit logs, and external bridges with zero external crate dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct TraceContext {
    /// 16-byte distributed trace identifier.
    pub trace_id: [u8; 16],
    /// 8-byte parent or current span identifier.
    pub span_id: [u8; 8],
    /// 8-bit trace options flags (bit 0 = recorded/sampled).
    pub trace_flags: u8,
}

impl TraceContext {
    /// Flag indicating that this trace is sampled / recorded.
    pub const SAMPLED: u8 = 0x01;

    /// Create a new trace context with explicit components.
    #[inline]
    pub const fn new(trace_id: [u8; 16], span_id: [u8; 8], trace_flags: u8) -> Self {
        Self {
            trace_id,
            span_id,
            trace_flags,
        }
    }

    /// Create a new sampled root trace context with given identifiers.
    #[inline]
    pub const fn new_root(trace_id: [u8; 16], span_id: [u8; 8]) -> Self {
        Self {
            trace_id,
            span_id,
            trace_flags: Self::SAMPLED,
        }
    }

    /// Derives a child span context carrying the same `trace_id` and `trace_flags`
    /// but with a new `span_id`.
    #[inline]
    pub const fn child_span(&self, new_span_id: [u8; 8]) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: new_span_id,
            trace_flags: self.trace_flags,
        }
    }

    /// Returns `true` if the recorded/sampled bit is set in `trace_flags`.
    #[inline]
    pub const fn is_sampled(&self) -> bool {
        (self.trace_flags & Self::SAMPLED) == Self::SAMPLED
    }

    /// Formats this trace context as a standard W3C `traceparent` HTTP header string.
    ///
    /// Specification format: `00-{trace_id_hex}-{span_id_hex}-{trace_flags_hex}`
    /// Example: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`
    pub fn to_traceparent(&self) -> String {
        format!(
            "00-{:032x}-{:016x}-{:02x}",
            u128::from_be_bytes(self.trace_id),
            u64::from_be_bytes(self.span_id),
            self.trace_flags
        )
    }

    /// Parses a standard W3C `traceparent` header string into a [`TraceContext`].
    pub fn from_traceparent(header: &str) -> Result<Self, CoreError> {
        let parts: Vec<&str> = header.trim().split('-').collect();
        if parts.len() != 4 {
            return Err(CoreError::Validation(format!(
                "invalid traceparent header: expected 4 hyphen-separated parts, found {}",
                parts.len()
            )));
        }

        let version = parts[0];
        if version != "00" {
            return Err(CoreError::Validation(format!(
                "unsupported traceparent version '{version}' (expected '00')"
            )));
        }

        let trace_id_str = parts[1];
        if trace_id_str.len() != 32 {
            return Err(CoreError::Validation(format!(
                "invalid trace_id length: expected 32 hex chars, found {}",
                trace_id_str.len()
            )));
        }
        let trace_id_val = u128::from_str_radix(trace_id_str, 16).map_err(|e| {
            CoreError::Validation(format!("invalid hex trace_id '{trace_id_str}': {e}"))
        })?;
        if trace_id_val == 0 {
            return Err(CoreError::Validation(
                "invalid trace_id: cannot be all zeros per W3C spec".to_string(),
            ));
        }

        let span_id_str = parts[2];
        if span_id_str.len() != 16 {
            return Err(CoreError::Validation(format!(
                "invalid span_id length: expected 16 hex chars, found {}",
                span_id_str.len()
            )));
        }
        let span_id_val = u64::from_str_radix(span_id_str, 16).map_err(|e| {
            CoreError::Validation(format!("invalid hex span_id '{span_id_str}': {e}"))
        })?;
        if span_id_val == 0 {
            return Err(CoreError::Validation(
                "invalid span_id: cannot be all zeros per W3C spec".to_string(),
            ));
        }

        let flags_str = parts[3];
        if flags_str.len() != 2 {
            return Err(CoreError::Validation(format!(
                "invalid trace_flags length: expected 2 hex chars, found {}",
                flags_str.len()
            )));
        }
        let trace_flags = u8::from_str_radix(flags_str, 16).map_err(|e| {
            CoreError::Validation(format!("invalid hex trace_flags '{flags_str}': {e}"))
        })?;

        Ok(Self {
            trace_id: trace_id_val.to_be_bytes(),
            span_id: span_id_val.to_be_bytes(),
            trace_flags,
        })
    }
}
