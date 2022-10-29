use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::p_set::*;
use crate::FinalPingDataT;
use crate::IcmpV4;
use crate::PingError;
use crate::PingResult;

pub(crate) struct PingReceiver<S> {
    states: Vec<State>,
    icmpv4: Arc<IcmpV4>,
    socket: Arc<S>,
    chan_rx: Option<crate::Receiver>,
    halt_tx: mpsc::Sender<()>,
    halt_rx: Option<mpsc::Receiver<()>>,
    thread_handle: Option<JoinHandle<()>>,
    results_tx: mpsc::SyncSender<PingResult<FinalPingDataT>>,
    results_rx: mpsc::Receiver<PingResult<FinalPingDataT>>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum State {
    New,
    Receiving,
    Halted,
}

impl<S> Drop for PingReceiver<S> {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            panic!("you must call halt on PingerReceiver to clean it up");
        }
    }
}

impl<S> PingReceiver<S>
where
    S: crate::Socket + 'static,
{
    pub(crate) fn new(
        icmpv4: Arc<IcmpV4>,
        socket: Arc<S>,
        chan_rx: crate::Receiver,
        channel_size: usize,
    ) -> Self {
        let (halt_tx, halt_rx) = mpsc::channel::<()>();
        let (results_tx, results_rx) =
            mpsc::sync_channel::<PingResult<FinalPingDataT>>(channel_size);
        PingReceiver {
            states: vec![State::New],
            icmpv4,
            socket,
            chan_rx: Some(chan_rx),
            halt_tx,
            halt_rx: Some(halt_rx),
            thread_handle: None,
            results_tx,
            results_rx,
        }
    }

    pub(crate) fn get_states(&self) -> Vec<State> {
        self.states.clone()
    }

    pub fn next_result(&self) -> PingResult<FinalPingDataT> {
        if *self.states.last().expect("logic error") == State::Halted {
            return Err(PingError {
                message: "cannot get next result when PingReceiver is halted".to_string(),
                source: None,
            }
            .into());
        }

        match self.results_rx.try_recv() {
            Err(e) => Err(e.into()),
            Ok(Err(e)) => Err(e),
            Ok(Ok(ok)) => Ok(ok),
        }
    }

    pub(crate) fn halt(mut self) -> std::thread::Result<()> {
        if *self.states.last().expect("logic error") == State::Halted {
            return Ok(());
        }

        let _ = self.halt_tx.send(());
        let join_result = match self.thread_handle.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };
        self.states.push(State::Halted);
        join_result
    }

    pub(crate) fn start(&mut self) {
        if *self.states.last().expect("logic error") != State::New {
            return;
        }

        let mut pset = PSet::new(self.chan_rx.take().expect("logic error"));

        let icmpv4 = self.icmpv4.clone();
        let socket = self.socket.clone();
        let results_tx = self.results_tx.clone();
        let halt_rx = self.halt_rx.take().expect("logic error");

        self.thread_handle = Some(std::thread::spawn(move || {
            'outer: loop {
                match halt_rx.try_recv() {
                    Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }

                let recv_echo_result = icmpv4.try_receive(&*socket);
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
                        let mut contains = pset.contains(&(ip, sn));
                        if !contains {
                            pset.update().unwrap();
                        }
                        contains = pset.contains(&(ip, sn));
                        pset.remove(&(ip, sn));
                        if !contains {
                            println!("log ERROR: on receive not contained");
                            break 'outer;
                        }
                        match results_tx.send(Ok((n, ip, sn))) {
                            Ok(()) => {}
                            Err(e) => {
                                println!("log ERROR: could not send notification");
                                break 'outer;
                            }
                        }
                    }
                }
            }
            println!("log TRACE: PingReceiver thread end");
        }));

        self.states.push(State::Receiving);
    }
}
