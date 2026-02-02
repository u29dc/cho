//! HTTP transport layer for the Xero API.
//!
//! Handles rate limiting, pagination, request building with auth/tenant
//! header injection, and response parsing with retry on 429.

pub mod pagination;
pub mod rate_limit;
pub mod request;
