//! cho-sdk: Xero accounting REST API client for Rust.
//!
//! A pure API client crate with zero CLI/TUI/MCP dependencies, publishable
//! to crates.io as a standalone Xero Rust SDK.
//!
//! # Architecture
//!
//! - [`client::XeroClient`] is the primary entry point for all API operations.
//! - [`auth`] handles OAuth 2.0 PKCE and client credentials flows.
//! - [`http`] manages rate limiting, pagination, and request building.
//! - [`models`] provides typed Rust structs for all Xero API resources.
//! - [`api`] provides namespaced API handles (e.g., `client.invoices()`).

#![deny(missing_docs)]

pub mod api;
pub mod auth;
pub mod blocking;
pub mod client;
pub mod config;
pub mod error;
pub mod http;
pub mod models;
