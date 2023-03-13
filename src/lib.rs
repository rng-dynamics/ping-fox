#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)] // TODO

pub use ping_fox::*;
pub use ping_receive::*;

use generic_error::GenericError;
use icmp::v4::IcmpV4;
use ping_data_buffer::PingDataBuffer;
use ping_error::PingError;
use ping_receiver::PingReceiver;
use ping_result::PingResult;
use ping_sender::PingSender;

mod generic_error;
mod icmp;
mod ping_data_buffer;
mod ping_error;
mod ping_fox;
mod ping_receive;
mod ping_receiver;
mod ping_result;
mod ping_sender;
mod records;
