#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub use ping_fox::*;
pub use ping_receive::*;

mod details;
mod ping_fox;
mod ping_receive;
