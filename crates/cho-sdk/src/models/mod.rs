//! Xero API data models.
//!
//! One file per resource, plus shared types in [`common`], large enums in [`enums`],
//! and date newtypes in [`dates`].

pub mod account;
pub mod bank_transaction;
pub mod budget;
pub mod common;
pub mod connection;
pub mod contact;
pub mod credit_note;
pub mod currency;
pub mod dates;
pub mod enums;
pub mod invoice;
pub mod item;
pub mod linked_transaction;
pub mod manual_journal;
pub mod organisation;
pub mod overpayment;
pub mod payment;
pub mod prepayment;
pub mod purchase_order;
pub mod quote;
pub mod repeating_invoice;
pub mod report;
pub mod tax_rate;
pub mod tracking_category;
