#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! cho-sdk: FreeAgent API client core.
//!
//! This crate provides authentication, transport, pagination, and a generic
//! resource API surface that powers the `cho` CLI.

pub mod api;
pub mod auth;
pub mod blocking;
pub mod client;
pub mod config;
pub mod error;
pub mod home;
pub mod liabilities;
pub mod models;
