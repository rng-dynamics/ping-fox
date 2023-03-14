#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub use ping_fox::*;
pub use ping_receive::*;

use generic_error::GenericError;
use ping_data_buffer::PingDataBuffer;
use ping_error::PingError;
use ping_receiver_details::PingReceiverDetails;
use ping_result::PingResult;
use ping_sender_details::PingSenderDetails;

mod generic_error;
mod icmp;
mod ping_data_buffer;
mod ping_error;
mod ping_fox;
mod ping_receive;
mod ping_receiver_details;
mod ping_result;
mod ping_sender_details;
mod records;
