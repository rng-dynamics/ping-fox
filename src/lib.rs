#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)] // TODO

pub use ping_output::*;
pub use ping_runner::*;
pub use ttl::Ttl;

pub(crate) use ping_receive_data::PingReceiveData;

use icmp::v4::IcmpV4;
use ping_data_buffer::PingDataBuffer;
use ping_error::{GenericError, PingError};
use ping_receiver::PingReceiver;
use ping_sender::PingSender;

mod event;
mod icmp;
mod ping_data_buffer;
mod ping_error;
mod ping_output;
mod ping_receive_data;
mod ping_receiver;
mod ping_runner;
mod ping_sender;
mod ttl;
