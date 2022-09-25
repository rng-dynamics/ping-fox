use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::p_set::*;
use crate::IcmpV4;
use crate::PingDataT;
use crate::PingResult;

pub(crate) struct PingReceiver {
    states: Vec<State>,
    icmpv4: Arc<IcmpV4>,
    socket: Arc<socket2::Socket>,
    sender_receiver_tx: mpsc::SyncSender<PSetDataT>,
    sender_receiver_rx: Option<mpsc::Receiver<PSetDataT>>,
    shutdown: mpsc::Sender<()>,
    is_shutdown: Option<mpsc::Receiver<()>>,
    thread_handle: Option<JoinHandle<()>>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum State {
    New,
    Receiving,
    Terminated,
}

impl Drop for PingReceiver {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            panic!("you must call shutdown on PingerReceiver to clean it up");
        }
    }
}

impl PingReceiver {
    pub(crate) fn new(
        icmpv4: Arc<IcmpV4>,
        socket: Arc<socket2::Socket>,
        sender_receiver_tx: mpsc::SyncSender<PSetDataT>,
        sender_receiver_rx: mpsc::Receiver<PSetDataT>,
    ) -> Self {
        let (shutdown, is_shutdown) = mpsc::channel();
        PingReceiver {
            states: vec![State::New],
            icmpv4,
            socket,
            sender_receiver_tx,
            sender_receiver_rx: Some(sender_receiver_rx),
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

    pub(crate) fn start(&mut self, tx: mpsc::SyncSender<PingResult<PingDataT>>) {
        if *self.states.last().expect("logic error") != State::New {
            return;
        }

        let sender_receiver_tx = self.sender_receiver_tx.clone();
        let sender_receiver_rx = self.sender_receiver_rx.take().expect("logic error");
        let mut pset = PSet::new(PSetSender::Sync(sender_receiver_tx), sender_receiver_rx);

        let icmpv4 = self.icmpv4.clone();
        let socket = self.socket.clone();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let is_shutdown = self.is_shutdown.take().expect("logic error");
        self.thread_handle = Some(std::thread::spawn(move || {
            'outer: loop {
                match is_shutdown.try_recv() {
                    Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }

                let recv_echo_result = icmpv4.try_receive(&socket);
                match recv_echo_result {
                    Ok(None) => {
                        println!("log TRACE: try_receive Ok(None)");
                        // nothing received
                        continue 'outer;
                    }
                    Err(e) => {
                        println!("log TRACE: try_receive Err(e)");
                        println!("log ERROR: error receiving icmp: {}", e);
                    }
                    Ok(Some((n, ip, sn))) => {
                        println!("log TRACE: try_receive Ok(Some((ip, sn)))");
                        println!("log TRACE: icmpv4 received");
                        if let IpAddr::V4(ipv4) = ip {
                            let mut contains = pset.contains(&(ipv4, sn));
                            if !contains {
                                pset.update().unwrap();
                            }
                            contains = pset.contains(&(ipv4, sn));
                            pset.remove(&(ipv4, sn));
                            if !contains {
                                println!("log ERROR: on receive not contained");
                                break 'outer;
                            }
                            match tx.send(Ok((n, ipv4, sn))) {
                                Ok(()) => {}
                                Err(e) => {
                                    println!("log ERROR: could not send notification");
                                    break 'outer;
                                }
                            }
                        } else {
                            println!("log ERROR: received non-V4");
                            panic!();
                        }
                    }
                }
            }
            println!("log TRACE: PingReceiver thread end");
        }));

        self.states.push(State::Receiving);
    }
}
