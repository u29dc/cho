//! Xero API data models.
//!
//! One file per resource, plus shared types in [`common`], large enums in [`enums`],
//! and date newtypes in [`dates`].

pub mod account;
pub mod bank_transaction;
pub mod common;
pub mod connection;
pub mod contact;
pub mod dates;
pub mod enums;
pub mod invoice;
pub mod payment;
pub mod report;
