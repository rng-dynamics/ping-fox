//! A ping (ICMP) library.
//!
//! Ping-fox is simple to use and it provides all features without elevated privileges.
//!
#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![warn(missing_docs)]

pub use ping_fox::*;
pub use ping_receive::*;

mod details;
mod ping_fox;
mod ping_receive;
