#![warn(rust_2018_idioms)]

use socket2::{Domain, Protocol, Type};
use std::collections::VecDeque;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

mod icmpv4;
mod p_set;
mod ping_error;
mod ping_receiver;
mod ping_sender;
mod socket;

use icmpv4::*;
use p_set::*;
use ping_error::*;
use ping_receiver::*;
use ping_sender::*;
use socket::*;

pub use ping_error::GenericError;

pub type PingResult<T> = std::result::Result<T, GenericError>;

// payload size, ip address, sequence number
type PingDataT = (usize, Ipv4Addr, u16);

pub struct Config {
    channel_size: usize,
}

impl Config {
    pub fn new(channel_size: usize) -> Config {
        Config { channel_size }
    }
}

fn create_socket(timeout: Duration) -> Result<socket2::Socket, GenericError> {
    // TODO: make UDP vs raw socket configurable
    let socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
    socket
        .set_read_timeout(Some(timeout))
        .expect("could not set socket timeout");
    Ok(socket)
}

pub struct Ping {
    pub receiver: std::sync::mpsc::Receiver<PingResult<PingDataT>>,
    ping_sender: PingSender<socket2::Socket>,
    ping_receiver: PingReceiver<socket2::Socket>,
}

impl Ping {
    pub fn create(config: &Config, ips: &[Ipv4Addr], count: u16) -> Self {
        let mut deque = VecDeque::<Ipv4Addr>::new();
        for ip in ips {
            deque.push_back(*ip);
        }
        let (sender_receiver_tx, sender_receiver_rx) =
            std::sync::mpsc::sync_channel::<PSetDataT>(config.channel_size);
        let (tx, rx) = std::sync::mpsc::sync_channel::<PingResult<PingDataT>>(config.channel_size);

        let icmpv4 = std::sync::Arc::new(IcmpV4::create());
        let socket = Arc::new(create_socket(Duration::from_millis(200)).unwrap()); // TODO(as): no unwrap

        let mut ping_sender =
            PingSender::new(icmpv4.clone(), socket.clone(), sender_receiver_tx.clone());
        let mut ping_receiver = PingReceiver::new(
            icmpv4.clone(),
            socket.clone(),
            sender_receiver_tx.clone(),
            sender_receiver_rx,
        );

        ping_sender.start(count, deque);
        ping_receiver.start(tx);

        Self {
            receiver: rx,
            ping_sender,
            ping_receiver,
        }
    }
    pub fn halt(self) -> std::thread::Result<()> {
        println!("ping.halt() calling ping_sender");
        let maybe_err_1 = self.ping_sender.halt();
        println!("ping.halt() calling ping_receiver");
        let maybe_err_2 = self.ping_receiver.halt();

        println!("ping.halt() checking errors");
        if maybe_err_1.is_err() {
            return Err(maybe_err_1.err().unwrap());
        }
        if maybe_err_2.is_err() {
            return Err(maybe_err_2.err().unwrap());
        }
        println!("ping.halt() done");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_localhost() {
        let config = Config::new(64);

        let ips = [Ipv4Addr::new(127, 0, 0, 1)];
        let ping: Ping = Ping::create(&config, &ips, 1); // ping_rs.ping(1);
        println!("ping.start_ping() done");

        let _ = ping.receiver.recv().unwrap();
        println!("in test received");
        // let (hostname, ip, dur) = pinger_thread.receiver.recv().unwrap().unwrap();
        // std::thread::sleep(Duration::from_secs(1));

        let halt_result = ping.halt();
        println!("pinger_trhead.halt() done");
        println!("end: {:?}", halt_result);
    }
}
