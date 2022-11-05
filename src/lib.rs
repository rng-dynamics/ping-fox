#![warn(rust_2018_idioms)]

use ping_output::ping_output_channel;
use socket2::{Domain, Protocol, Type};
use std::collections::VecDeque;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

mod icmpv4;
// mod p_set;
mod event;
mod ping_data_buffer;
mod ping_error;
mod ping_output;
mod ping_receiver;
mod ping_sender;
mod ping_sent_sync_event;
mod socket;

use icmpv4::*;
// use p_set::*;
use event::*;
use ping_data_buffer::*;
use ping_error::*;
use ping_output::*;
use ping_receiver::*;
use ping_sender::*;
use ping_sent_sync_event::*;
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

    ping_data_buffer: PingDataBuffer,

    ping_output_rx: PingOutputReceiver,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum State {
    Running,
    Halted,
}

const CHANNEL_SIZE: usize = 8; // TODO: config

impl Ping {
    pub fn start(config: &Config, ips: &[Ipv4Addr], count: u16) -> Self {
        let mut deque = VecDeque::<Ipv4Addr>::new();
        for ip in ips {
            deque.push_back(*ip);
        }

        // TODO(as): no unwrap
        let icmpv4 = std::sync::Arc::new(IcmpV4::create());
        let socket = Arc::new(create_socket(Duration::from_millis(200)).unwrap());
        let (send_sync_tx, send_sync_rx) = ping_send_sync_event_channel();
        let (receive_event_tx, receive_event_rx) = ping_receive_event_channel();
        let (send_event_tx, send_event_rx) = ping_send_event_channel();
        let (ping_output_tx, ping_output_rx) = ping_output_channel();

        let mut ping_sender =
            PingSender::new(icmpv4.clone(), socket.clone(), send_event_tx, send_sync_tx);
        let mut ping_receiver =
            PingReceiver::new(icmpv4, socket, send_sync_rx, receive_event_tx, CHANNEL_SIZE);
        let ping_data_buffer = PingDataBuffer::new(send_event_rx, receive_event_rx, ping_output_tx);

        ping_sender.start(count, deque.into());
        ping_receiver.start();

        Self {
            states: vec![State::Running],
            ping_sender,
            ping_receiver,
            ping_data_buffer,
            ping_output_rx,
        }
    }

    // pub fn start(config: &Config, ips: &[Ipv4Addr], count: u16) -> Self {
    //     let mut deque = VecDeque::<Ipv4Addr>::new();
    //     for ip in ips {
    //         deque.push_back(*ip);
    //     }

    //     let icmpv4 = std::sync::Arc::new(IcmpV4::create());
    //     let socket = Arc::new(create_socket(Duration::from_millis(200)).unwrap()); // TODO(as): no unwrap
    //     let (thread_comm_tx, thread_comm_rx) = create_sync_channel(config.channel_size);

    //     let mut ping_receiver = PingReceiver::new(
    //         icmpv4.clone(),
    //         socket.clone(),
    //         thread_comm_rx,
    //         config.channel_size,
    //     );
    //     ping_receiver.start();
    //     let mut ping_sender = PingSender::new(icmpv4.clone(), socket.clone(), thread_comm_tx);
    //     ping_sender.start(count, deque);

    //     Self {
    //         states: vec![State::Running],
    //         ping_sender,
    //         ping_receiver,
    //     }
    // }

    pub(crate) fn get_states(&self) -> Vec<State> {
        self.states.clone()
    }

    // TODO
    pub fn next_output(&mut self) -> PingResult<PingOutput> {
        self.ping_data_buffer.process();
        Ok(self.ping_output_rx.recv().unwrap())
    }

    pub fn halt(mut self) -> std::thread::Result<()> {
        if *self.states.last().expect("logic error") == State::Halted {
            return Ok(());
        }
        println!("ping.halt() calling ping_sender");
        let maybe_err_1 = self.ping_sender.halt();
        drop(self.ping_sender);
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
        let mut ping: Ping = Ping::start(&config, &ips, 1);
        println!("ping.start_ping() done");

        std::thread::sleep(std::time::Duration::from_secs(1));
        let output = ping.next_output().unwrap();
        println!("output received: {:?}", output);

        let halt_result = ping.halt();
        println!("pinger_trhead.halt() done");
        println!("end: {:?}", halt_result);
    }
}
