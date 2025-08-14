#![allow(clippy::identity_op)]
mod bindings;
mod entry;
mod error;

use entry::Component;

bindings::export!(Component with_types_in crate::bindings);