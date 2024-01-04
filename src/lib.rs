
#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

pub mod sen5x;

mod error;
pub use error::Error;

mod commands;
pub use commands::Command;

pub mod types;
pub use types::SensorDataInt;

mod utils;