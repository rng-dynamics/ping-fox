use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use socket2::{Domain, Protocol, Type};

use crate::*;

pub use crate::ping_error::GenericError;

pub type PingResult<T> = std::result::Result<T, GenericError>;

// payload size, ip address, sequence number
// type FinalPingDataT = (usize, IpAddr, u16);

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

#[derive(Clone, PartialEq, Debug)]
pub enum State {
    // TODO(as): add a state 'New'
    Running,
    Halted,
}

pub struct PingRunner {
    states: Vec<State>,
    sender_halt_tx: mpsc::Sender<()>,
    sender_thread: Option<JoinHandle<()>>,
    receiver_halt_tx: mpsc::Sender<()>,
    receiver_thread: Option<JoinHandle<()>>,
    ping_output_rx: PingOutputReceiver,
}

impl PingRunner {
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
        let ping_receiver = PingReceiver::new(icmpv4, socket, receive_event_tx);
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

    pub fn next_ping_output(&self) -> PingResult<PingOutput> {
        Ok(self.ping_output_rx.recv().unwrap())
    }

    pub fn halt(mut self) -> std::thread::Result<()> {
        if *self.states.last().expect("logic error") == State::Halted {
            return Ok(());
        }
        // mpsc::Sender::send() returns error only if mpsc::Receiver is closed.
        let _maybe_err_1 = self.sender_halt_tx.send(());
        let _maybe_err_2 = self.receiver_halt_tx.send(());

        let join_result_1 = match self.sender_thread.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };
        let join_result_2 = match self.receiver_thread.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };

        if let Err(e) = join_result_1 {
            return Err(e.into());
        }
        println!("ping.halt() 6");
        if let Err(e) = join_result_2 {
            return Err(e.into());
        }

        println!("ping.halt() done");
        self.states.push(State::Halted);
        Ok(())
    }

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
                let receive_result = ping_receiver.receive();
                if let Err(_) = receive_result {
                    println!("log ERROR: PingReceiver::receive() failed");
                    break 'outer;
                }
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
                    if ping_sender.send_one(*ip, sequence_number).is_err() {
                        println!("log ERROR: PingSender::send_one() failed");
                        break 'outer;
                    }
                    // (2.2) Dispatch sync event.
                    if ping_send_sync_event_tx.send(PingSentSyncEvent).is_err() {
                        println!("log ERROR: mpsc::SyncSender::send() failed");
                        break 'outer;
                    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_localhost() {
        let config = Config::new(64);

        let ips = [Ipv4Addr::new(127, 0, 0, 1)];
        let ping: PingRunner = PingRunner::start(&config, &ips, 1);
        println!("ping.start_ping() done");

        let output = ping.next_ping_output().unwrap();
        println!("output received: {:?}", output);

        let halt_result = ping.halt();
        println!("pinger_thead.halt() done");
        println!("end: {:?}", halt_result);
    }
}
