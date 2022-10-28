#![warn(rust_2018_idioms)]

use socket2::{Domain, Protocol, Type};
use std::collections::VecDeque;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

mod channel;
mod icmpv4;
mod p_set;
mod ping_error;
mod ping_receiver;
mod ping_sender;
mod socket;

use channel::*;
use icmpv4::*;
use p_set::*;
use ping_error::*;
use ping_receiver::*;
use ping_sender::*;
use socket::*;

pub use ping_error::GenericError;

pub type PingResult<T> = std::result::Result<T, GenericError>;

type InternalData = (usize, IpAddr, u16, Instant);

// payload size, ip address, sequence number
type FinalPingDataT = (usize, IpAddr, u16);

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
    states: Vec<State>,
    // rx: mpsc::Receiver<PingResult<FinalPingDataT>>,
    ping_sender: PingSender<socket2::Socket>,
    ping_receiver: PingReceiver<socket2::Socket>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum State {
    Running,
    Halted,
}

impl Ping {
    pub fn create(config: &Config, ips: &[Ipv4Addr], count: u16) -> Self {
        let mut deque = VecDeque::<Ipv4Addr>::new();
        for ip in ips {
            deque.push_back(*ip);
        }

        let icmpv4 = std::sync::Arc::new(IcmpV4::create());
        let socket = Arc::new(create_socket(Duration::from_millis(200)).unwrap()); // TODO(as): no unwrap
        let (thread_comm_tx, thread_comm_rx) = create_sync_channel(config.channel_size);

        let mut ping_receiver = PingReceiver::new(
            icmpv4.clone(),
            socket.clone(),
            thread_comm_rx,
            config.channel_size,
        );
        ping_receiver.start();
        let mut ping_sender = PingSender::new(icmpv4.clone(), socket.clone(), thread_comm_tx);
        ping_sender.start(count, deque);

        Self {
            states: vec![State::Running],
            ping_sender,
            ping_receiver,
        }
    }

    pub(crate) fn get_states(&self) -> Vec<State> {
        self.states.clone()
    }

    pub fn next_result(&self) -> PingResult<FinalPingDataT> {
        self.ping_receiver.next_result()
    }

    pub fn halt(mut self) -> std::thread::Result<()> {
        if *self.states.last().expect("logic error") == State::Halted {
            return Ok(());
        }
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
        self.states.push(State::Halted);
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
        let ping: Ping = Ping::create(&config, &ips, 1);
        println!("ping.start_ping() done");

        std::thread::sleep(std::time::Duration::from_secs(1));
        let _ = ping.next_result().unwrap();
        println!("in test received");

        let halt_result = ping.halt();
        println!("pinger_trhead.halt() done");
        println!("end: {:?}", halt_result);
    }
}
