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
            panic!("you must call halt on PingReceiver to clean it up");
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

    pub(crate) fn halt(&mut self) -> std::thread::Result<()> {
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
                let recv_echo_result = icmpv4.try_receive(&*socket);
                match recv_echo_result {
                    Ok(None) => {
                        println!("log TRACE: try_receive Ok(None)");
                        // nothing received
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
                            Err(_) => {
                                println!("log ERROR: could not send notification");
                                break 'outer;
                            }
                        }
                        println!("log TRACE: sent received result to output-channel");
                    }
                }
                match halt_rx.try_recv() {
                    Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }
            }
            println!("log TRACE: PingReceiver thread end");
        }));

        println!("log TRACE: PingReceiver started");
        self.states.push(State::Receiving);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::tests::OnReceive;
    use crate::socket::tests::OnSend;
    use crate::socket::tests::SocketMock;

    use std::net::Ipv4Addr;

    const CHANNEL_SIZE: usize = 8;

    #[test]
    fn entity_states() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (_comm_chan_tx, comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, comm_chan_rx, CHANNEL_SIZE);

        assert!(vec![State::New] == ping_receiver.get_states());
        ping_receiver.start();
        assert!(vec![State::New, State::Receiving] == ping_receiver.get_states());
        let _ = ping_receiver.halt();
        assert!(vec![State::New, State::Receiving, State::Halted] == ping_receiver.get_states());
    }

    #[test]
    #[should_panic(expected = "you must call halt on PingReceiver to clean it up")]
    fn not_calling_halt_may_panic_on_drop() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (_comm_chan_tx, comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, comm_chan_rx, CHANNEL_SIZE);
        ping_receiver.start();

        drop(ping_receiver);
    }

    #[test]
    fn receive_ping_packets_success() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(1),
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (comm_chan_tx, comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);
        let payload_size = 4;
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let sequence_number = 0;
        let send_time = std::time::Instant::now();

        comm_chan_tx
            .send((payload_size, ip_addr, sequence_number, send_time))
            .unwrap();

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, comm_chan_rx, CHANNEL_SIZE);
        ping_receiver.start();
        // TODO: get rid of that sleep
        std::thread::sleep(std::time::Duration::from_millis(1));
        println!("log TRACE: receive_ping_packets_success: will call next_result");
        let next_ping_receiver_result = ping_receiver.next_result();
        println!("log TRACE: receive_ping_packets_success: call next_result done");

        let _ = ping_receiver.halt();

        assert!(next_ping_receiver_result.is_ok());
    }
}
