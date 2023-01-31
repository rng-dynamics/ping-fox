#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)] // TODO

mod c_dgram_socket;
mod event;
mod icmpv4;
mod icmpv4_socket;
mod ping_data_buffer;
mod ping_error;
mod ping_output;
mod ping_receiver;
mod ping_runner;
mod ping_sender;
mod socket;

use icmpv4::IcmpV4;
use ping_data_buffer::PingDataBuffer;
use ping_error::{GenericError, PingError};
pub use ping_output::*;
use ping_receiver::PingReceiver;
pub use ping_runner::*;
use ping_sender::PingSender;
use socket::Socket;
