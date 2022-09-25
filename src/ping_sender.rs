use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::icmpv4::IcmpV4;
use crate::p_set::PSetDataT;

pub(crate) struct PingSender {
    states: Vec<State>,
    icmpv4: Arc<IcmpV4>,
    socket: Arc<socket2::Socket>,
    sender_receiver_tx: mpsc::SyncSender<PSetDataT>,
    shutdown: mpsc::Sender<()>,
    is_shutdown: Option<mpsc::Receiver<()>>,
    thread_handle: Option<JoinHandle<()>>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum State {
    New,
    Sending,
    Terminated,
}

impl Drop for PingSender {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            panic!("you must call shutdown on PingSender to clean it up");
        }
    }
}

impl PingSender {
    pub(crate) fn new(
        icmpv4: Arc<IcmpV4>,
        socket: Arc<socket2::Socket>,
        sender_receiver_tx: mpsc::SyncSender<PSetDataT>,
    ) -> Self {
        let (shutdown, is_shutdown) = mpsc::channel();
        PingSender {
            states: vec![State::New],
            icmpv4,
            socket,
            sender_receiver_tx,
            shutdown,
            is_shutdown: Some(is_shutdown),
            thread_handle: None,
        }
    }

    pub(crate) fn get_states(&self) -> Vec<State> {
        self.states.clone()
    }

    pub(crate) fn shutdown(mut self) -> std::thread::Result<()> {
        if *self.states.last().expect("logic error") == State::Terminated {
            return Ok(());
        }
        let _ = self.shutdown.send(());
        let join_result = match self.thread_handle.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };
        self.states.push(State::Terminated);
        join_result
    }

    pub(crate) fn start(&mut self, count: u16, ips: VecDeque<Ipv4Addr>) {
        if *self.states.last().expect("logic error") != State::New {
            return;
        }

        let icmpv4 = self.icmpv4.clone();
        let socket = self.socket.clone();
        let sender_receiver_tx = self.sender_receiver_tx.clone();
        let is_shutdown = self.is_shutdown.take().expect("logic error");

        self.thread_handle = Some(std::thread::spawn(move || {
            println!("log TRACE: PingSender thread start");
            'outer: for sequence_number in 0..count {
                println!("log TRACE: PingSender outer loop start");
                for ip in &ips {
                    println!("log TRACE: PingSender inner loop start");
                    let send_echo_result = icmpv4.send_one_ping(&socket, ip, sequence_number);
                    println!("log TRACE: ping sent");
                    if let Err(error) = send_echo_result {
                        println!("log ERROR: error sending one ping: {}", error);
                        break 'outer;
                    }
                    println!("log TRACE: icmpv4 successfully sent");

                    let payload_size = send_echo_result.unwrap().0;
                    sender_receiver_tx.send((*ip, sequence_number)).unwrap(); // TODO
                    println!("log TRACE: PingSender sent to PingReceiver");

                    match is_shutdown.try_recv() {
                        Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    }
                }
            }
            println!("log TRACE: PingSender thread end");
        }));

        self.states.push(State::Sending);
    }
}
