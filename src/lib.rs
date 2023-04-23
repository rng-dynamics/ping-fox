//! A ping (ICMP) library - simple to use and no root or setuid required.
//!
//! The following example describes how to configure ping-fox and how to send and
//! receive an echo messages and its response.
//!
//! ```
//! use ping_fox::{PingFoxConfig, PingReceive, PingReceiveData, PingSentToken, SocketType};
//! use std::net::Ipv4Addr;
//! use std::time::Duration;
//!
//! // ### Configure the library:
//! // - `socket_type` can be `SocketType::RAW` or `SocketType::DGRAM`.
//! // - Use `SocketType::DGRAM` to avoid the need for elevated privileges.
//! let config = PingFoxConfig {
//!     socket_type: SocketType::DGRAM,
//!     timeout: Duration::from_secs(1),
//!     channel_size: 1,
//! };
//!
//! // ### Create a ping sender and a ping receiver.
//! let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config).unwrap();
//!
//! // ### Call `PingSender::send_to`
//! let token: PingSentToken = ping_sender
//!     .send_to("127.0.0.1".parse::<Ipv4Addr>().unwrap())
//!     .unwrap();
//!
//! // ### Use the `PingSentToken` to call `PingReceiver::receive`.
//! let ping_response = ping_receiver.receive(token).unwrap();
//!
//! match ping_response {
//!     PingReceive::Data(PingReceiveData {
//!         package_size,
//!         ip_addr,
//!         ttl,
//!         sequence_number,
//!         ping_duration,
//!     }) => {
//!         println!(
//!             "{package_size} bytes from {ip_addr}: \
//!               icmp_seq={sequence_number} ttl={ttl} \
//!               time={ping_duration:?}",
//!         );
//!     }
//!     PingReceive::Timeout => {
//!         println!("timeout");
//!     }
//! };
//! ```
//!
#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![warn(missing_docs)]

pub use crate::ping_fox::*;
pub use ping_receive::*;

mod details;
mod ping_fox;
mod ping_receive;
