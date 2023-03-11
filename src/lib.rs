#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)] // TODO

pub use ping_fox::*;
pub use ping_response_data::*;

pub mod icmp;

use icmp::v4::IcmpV4;
use ping_data_buffer::PingDataBuffer;
use ping_error::{GenericError, PingError};
use ping_receive_data::PingReceiveData;
use ping_receiver::PingReceiver;
use ping_sender::PingSender;

mod event;
mod ping_data_buffer;
mod ping_error;
mod ping_fox;
mod ping_receive_data;
mod ping_receiver;
mod ping_response_data;
mod ping_sender;
