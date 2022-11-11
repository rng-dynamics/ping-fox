#![warn(rust_2018_idioms)]

use ping_output::ping_output_channel;
use socket2::{Domain, Protocol, Type};
use std::collections::VecDeque;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
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

// type InternalData = (usize, IpAddr, u16, Instant);

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

// pub struct Ping {
//     states: Vec<State>,
//     // rx: mpsc::Receiver<PingResult<FinalPingDataT>>,
//     ping_sender: PingSender<socket2::Socket>,
//     ping_receiver: PingReceiver<socket2::Socket>,
//
//     ping_data_buffer: PingDataBuffer,
//
//     ping_output_rx: PingOutputReceiver,
// }

#[derive(Clone, PartialEq, Debug)]
pub enum State {
    // TODO(as): add a state 'New'
    Running,
    Halted,
}

const CHANNEL_SIZE: usize = 8; // TODO: config

// impl Ping {
//     pub fn start(config: &Config, ips: &[Ipv4Addr], count: u16) -> Self {
//         let mut deque = VecDeque::<Ipv4Addr>::new();
//         for ip in ips {
//             deque.push_back(*ip);
//         }
//
//         // TODO(as): no unwrap
//         let icmpv4 = std::sync::Arc::new(IcmpV4::create());
//         let socket = Arc::new(create_socket(Duration::from_millis(200)).unwrap());
//         let (send_sync_tx, send_sync_rx) = ping_send_sync_event_channel();
//         let (receive_event_tx, receive_event_rx) = ping_receive_event_channel();
//         let (send_event_tx, send_event_rx) = ping_send_event_channel();
//         let (ping_output_tx, ping_output_rx) = ping_output_channel();
//
//         let mut ping_sender =
//             PingSender::new(icmpv4.clone(), socket.clone(), send_event_tx, send_sync_tx);
//         let mut ping_receiver =
//             PingReceiver::new(icmpv4, socket, send_sync_rx, receive_event_tx, CHANNEL_SIZE);
//         let ping_data_buffer = PingDataBuffer::new(send_event_rx, receive_event_rx, ping_output_tx);
//
//         ping_sender.start(count, deque.into());
//         ping_receiver.start();
//
//         Self {
//             states: vec![State::Running],
//             ping_sender,
//             ping_receiver,
//             ping_data_buffer,
//             ping_output_rx,
//         }
//     }
//
//     // pub fn start(config: &Config, ips: &[Ipv4Addr], count: u16) -> Self {
//     //     let mut deque = VecDeque::<Ipv4Addr>::new();
//     //     for ip in ips {
//     //         deque.push_back(*ip);
//     //     }
//
//     //     let icmpv4 = std::sync::Arc::new(IcmpV4::create());
//     //     let socket = Arc::new(create_socket(Duration::from_millis(200)).unwrap()); // TODO(as): no unwrap
//     //     let (thread_comm_tx, thread_comm_rx) = create_sync_channel(config.channel_size);
//
//     //     let mut ping_receiver = PingReceiver::new(
//     //         icmpv4.clone(),
//     //         socket.clone(),
//     //         thread_comm_rx,
//     //         config.channel_size,
//     //     );
//     //     ping_receiver.start();
//     //     let mut ping_sender = PingSender::new(icmpv4.clone(), socket.clone(), thread_comm_tx);
//     //     ping_sender.start(count, deque);
//
//     //     Self {
//     //         states: vec![State::Running],
//     //         ping_sender,
//     //         ping_receiver,
//     //     }
//     // }
//
//     pub(crate) fn get_states(&self) -> Vec<State> {
//         self.states.clone()
//     }
//
//     // TODO
//     pub fn next_output(&mut self) -> PingResult<PingOutput> {
//         self.ping_data_buffer.process();
//         Ok(self.ping_output_rx.recv().unwrap())
//     }
//
//     pub fn halt(mut self) -> std::thread::Result<()> {
//         if *self.states.last().expect("logic error") == State::Halted {
//             return Ok(());
//         }
//         println!("ping.halt() calling ping_sender");
//         let maybe_err_1 = self.ping_sender.halt();
//         drop(self.ping_sender);
//         println!("ping.halt() calling ping_receiver");
//         let maybe_err_2 = self.ping_receiver.halt();
//
//         println!("ping.halt() checking errors");
//         if maybe_err_1.is_err() {
//             return Err(maybe_err_1.err().unwrap());
//         }
//         if maybe_err_2.is_err() {
//             return Err(maybe_err_2.err().unwrap());
//         }
//         println!("ping.halt() done");
//         self.states.push(State::Halted);
//         Ok(())
//     }
// }

// TODO(as): rename to PingRunner or so.
pub struct Ping {
    states: Vec<State>,
    // rx: mpsc::Receiver<PingResult<FinalPingDataT>>,
    // ping_sender: PingSender<socket2::Socket>,
    // ping_receiver: PingReceiver<socket2::Socket>,
    sender_halt_tx: mpsc::Sender<()>,
    // sender_halt_rx: Option<mpsc::Receiver<()>>,
    sender_thread: Option<JoinHandle<()>>,

    receiver_halt_tx: mpsc::Sender<()>,
    // receiver_halt_rx: Option<mpsc::Receiver<()>>,
    receiver_thread: Option<JoinHandle<()>>,

    // ping_data_buffer: PingDataBuffer,
    ping_output_rx: PingOutputReceiver,
}

impl Ping {
    fn start_receiver_thread(
        mut ping_data_buffer: PingDataBuffer,
        ping_receiver: PingReceiver<socket2::Socket>,
        halt_rx: mpsc::Receiver<()>,
        ping_send_sync_event_rx: mpsc::Receiver<PingSentSyncEvent>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            'outer: loop {
                // (1) Wait for sync-event from PingSender.
                let ping_sent_sync_event_recv = ping_send_sync_event_rx.recv();

                if let Err(_) = ping_sent_sync_event_recv {
                    println!("log INFO: mpsc::Receiver::recv() failed");
                    break 'outer;
                }

                // (2) receive ping and update ping buffer
                ping_receiver.recv();
                ping_data_buffer.update();

                // (4) check termination
                match halt_rx.try_recv() {
                    Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }
            }
        })
    }

    fn start_sender_thread(
        ping_sender: PingSender<socket2::Socket>,
        halt_rx: mpsc::Receiver<()>,
        count: u16,
        ips: VecDeque<Ipv4Addr>,
        ping_send_sync_event_tx: mpsc::SyncSender<PingSentSyncEvent>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            println!("log TRACE: PingSender thread start with count {}", count);
            'outer: for sequence_number in 0..count {
                println!("log TRACE: PingSender outer loop start");
                for ip in &ips {
                    println!("log TRACE: PingSender inner loop start");
                    ping_sender.send_one(*ip, sequence_number);
                    // (2.2) Dispatch sync event.
                    ping_send_sync_event_tx.send(PingSentSyncEvent);
                    println!("log TRACE: PingSender published SYNC-Event");

                    // (3) Check termination.
                    match halt_rx.try_recv() {
                        Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    }

                    // (4) TODO: Sleep according to configuration
                }
            }
            println!("log TRACE: PingSender thread end");
        })
    }

    pub fn start(config: &Config, ips: &[Ipv4Addr], count: u16) -> Self {
        let mut deque = VecDeque::<Ipv4Addr>::new();
        for ip in ips {
            deque.push_back(*ip);
        }

        // TODO(as): no unwrap
        let icmpv4 = std::sync::Arc::new(IcmpV4::create());
        let socket = Arc::new(create_socket(Duration::from_millis(200)).unwrap());

        let (send_sync_event_tx, send_sync_event_rx) = ping_send_sync_event_channel();
        let (receive_event_tx, receive_event_rx) = ping_receive_event_channel();
        let (send_event_tx, send_event_rx) = ping_send_event_channel();
        let (ping_output_tx, ping_output_rx) = ping_output_channel();

        let ping_sender = PingSender::new(icmpv4.clone(), socket.clone(), send_event_tx);
        let ping_receiver = PingReceiver::new(icmpv4, socket, receive_event_tx, CHANNEL_SIZE);
        let ping_data_buffer = PingDataBuffer::new(send_event_rx, receive_event_rx, ping_output_tx);

        let (sender_halt_tx, sender_halt_rx) = mpsc::channel::<()>();
        let sender_thread = Self::start_sender_thread(
            ping_sender,
            sender_halt_rx,
            count,
            deque.into(),
            send_sync_event_tx,
        );

        let (receiver_halt_tx, receiver_halt_rx) = mpsc::channel::<()>();
        let receiver_thread = Self::start_receiver_thread(
            ping_data_buffer,
            ping_receiver,
            receiver_halt_rx,
            send_sync_event_rx,
        );

        Self {
            states: vec![State::Running],

            sender_halt_tx,
            sender_thread: Some(sender_thread),

            receiver_halt_tx,
            receiver_thread: Some(receiver_thread),

            ping_output_rx,
        }
    }

    pub fn get_states(&self) -> Vec<State> {
        self.states.clone()
    }

    // TODO
    pub fn next_output(&self) -> PingResult<PingOutput> {
        Ok(self.ping_output_rx.recv().unwrap())
    }

    pub fn halt(mut self) -> std::thread::Result<()> {
        if *self.states.last().expect("logic error") == State::Halted {
            return Ok(());
        }
        println!("ping.halt() 0");
        // mpsc::Sender::send returns error only if mpsc::Receiver is closed.
        let _maybe_err_1 = self.sender_halt_tx.send(());
        let _maybe_err_2 = self.receiver_halt_tx.send(());

        // println!("ping.halt() 1");
        // if let Err(e) = maybe_err_1 {
        //     return Err(Box::new(e));
        // }
        // println!("ping.halt() 2");
        // if let Err(e) = maybe_err_2 {
        //     return Err(Box::new(e));
        // }

        println!("ping.halt() 3");
        let join_result_1 = match self.sender_thread.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };
        println!("ping.halt() 4");
        let join_result_2 = match self.receiver_thread.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };

        println!("ping.halt() 5");
        if let Err(e) = join_result_1 {
            return Err(Box::new(e));
        }
        println!("ping.halt() 6");
        if let Err(e) = join_result_2 {
            return Err(Box::new(e));
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

        let output = ping.next_output().unwrap();
        println!("output received: {:?}", output);

        let halt_result = ping.halt();
        println!("pinger_thead.halt() done");
        println!("end: {:?}", halt_result);
    }
}
