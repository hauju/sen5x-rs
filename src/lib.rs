//! This library provides an embedded `no_std` driver for the [Sensirion SEN5x series](https://sensirion.com/products/catalog/SEK-SEN5x).
//! This driver was built using [`embedded-hal`](https://docs.rs/embedded-hal/) traits.
//! The implementation is based on [embedded-i2c-sen5x](https://github.com/Sensirion/embedded-i2c-sen5x)
//! and [sgpc3-rs](https://github.com/mjaakkol/sgpc3-rs).
//!
//! ## `embedded-hal-async` Support
//!
//! This crate has optional support for the [`embedded-hal-async`] crate. The
//! [`Sen5xAsync`] type provides a driver for a SEN5x sensor which uses
//! [`embedded-hal-async`]'s asynchronous versions of the `I2c` and `DelayNs`
//! traits, rather than the blocking versions from [`embedded-hal`].
//!
//! The [`embedded-hal-async`] support is feature flagged, so that users who
//! don't need the asynchronous versions of these traits don't have to depend on
//! `embedded-hal-async`. To use it, enable the `embedded-hal-async` feature
//! flag in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sen5x = { version = "0.1", features = ["embedded-hal-async"] }
//! ```

#![deny(unsafe_code)]
#![cfg_attr(not(any(test, feature = "std")), no_std)]

mod sen5x;
pub use crate::sen5x::Sen5x;

#[cfg(feature = "embedded-hal-async")]
pub use crate::sen5x::Sen5xAsync;

mod error;
pub use error::Error;

pub mod commands;

pub mod types;
