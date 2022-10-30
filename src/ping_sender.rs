use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::icmpv4::IcmpV4;
use crate::p_set::PSetDataT;

pub(crate) struct PingSender<S> {
    states: Vec<State>,
    icmpv4: Arc<IcmpV4>,
    socket: Arc<S>,
    comm_chan_tx: crate::SyncSender,
    halt_tx: mpsc::Sender<()>,
    halt_rx: Option<mpsc::Receiver<()>>,
    thread_handle: Option<JoinHandle<()>>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum State {
    New,
    Sending,
    Halted,
}

impl<S> Drop for PingSender<S> {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            panic!("you must call halt on PingSender to clean it up");
        }
    }
}

impl<S> PingSender<S>
where
    S: crate::Socket + 'static,
{
    pub(crate) fn new(
        icmpv4: Arc<IcmpV4>,
        socket: Arc<S>,
        comm_chan_tx: crate::SyncSender,
    ) -> Self {
        let (halt_tx, halt_rx) = mpsc::channel::<()>();
        PingSender {
            states: vec![State::New],
            icmpv4,
            socket,
            comm_chan_tx,
            halt_tx,
            halt_rx: Some(halt_rx),
            thread_handle: None,
        }
    }

    pub(crate) fn get_states(&self) -> Vec<State> {
        self.states.clone()
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

    pub(crate) fn start(&mut self, count: u16, ips: VecDeque<Ipv4Addr>) {
        if *self.states.last().expect("logic error") != State::New {
            return;
        }

        let icmpv4 = self.icmpv4.clone();
        let socket = self.socket.clone();
        let comm_chan = self.comm_chan_tx.clone();
        let halt_rx = self.halt_rx.take().expect("logic error");

        self.thread_handle = Some(std::thread::spawn(move || {
            println!("log TRACE: PingSender thread start with count {}", count);
            'outer: for sequence_number in 0..count {
                println!("log TRACE: PingSender outer loop start");
                for ip in &ips {
                    println!("log TRACE: PingSender inner loop start");
                    let send_echo_result = icmpv4.send_one_ping(&*socket, ip, sequence_number);
                    println!("log TRACE: ping sent");
                    if let Err(error) = send_echo_result {
                        println!("log ERROR: error sending one ping: {}", error);
                        break 'outer;
                    }
                    println!("log TRACE: icmpv4 successfully sent");

                    let (payload_size, ip_addr, _, send_time) = send_echo_result.unwrap();
                    let comm_chan_send_result =
                        comm_chan.send((payload_size, ip_addr, sequence_number, send_time));
                    if let Err(error) = comm_chan_send_result {
                        println!(
                            "log ERROR: error sending internal data on channel: {}",
                            error
                        );
                    }
                    println!("log TRACE: PingSender sent to PingReceiver");

                    match halt_rx.try_recv() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::tests::OnReceive;
    use crate::socket::tests::OnSend;
    use crate::socket::tests::SocketMock;

    const CHANNEL_SIZE: usize = 8;

    #[test]
    fn entity_states() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (comm_chan_tx, _comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_sender = PingSender::new(icmpv4, socket_mock, comm_chan_tx);

        assert!(vec![State::New] == ping_sender.get_states());
        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(1, [ip_127_0_0_1].into());
        assert!(vec![State::New, State::Sending] == ping_sender.get_states());
        let _ = ping_sender.halt();
        assert!(vec![State::New, State::Sending, State::Halted] == ping_sender.get_states());
    }

    #[test]
    #[should_panic(expected = "you must call halt on PingSender to clean it up")]
    fn not_calling_halt_may_panic_on_drop() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (comm_chan_tx, _comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_sender = PingSender::new(icmpv4, socket_mock, comm_chan_tx);
        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(1, [ip_127_0_0_1].into());

        drop(ping_sender);
    }

    #[test]
    fn send_ping_packets_success() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (comm_chan_tx, comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_sender = PingSender::new(icmpv4, socket_mock, comm_chan_tx);
        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(2, [ip_127_0_0_1].into());

        let send_record_1 = comm_chan_rx.receive();
        let send_record_2 = comm_chan_rx.receive();
        let _ = ping_sender.halt();

        assert!(send_record_1.is_ok());
        let (_payload_size, ip_addr, sequence_number, _time) = send_record_1.unwrap();
        assert!(ip_127_0_0_1 == ip_addr);
        assert!(sequence_number == 0);

        assert!(send_record_2.is_ok());
        let (_payload_size, ip_addr, sequence_number, _time) = send_record_2.unwrap();
        assert!(ip_127_0_0_1 == ip_addr);
        assert!(sequence_number == 1);
    }

    #[test]
    fn when_socket_fails_then_ping_sender_fails() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (comm_chan_tx, comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_sender = PingSender::new(icmpv4, socket_mock, comm_chan_tx);
        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(1, [ip_127_0_0_1].into());

        assert!(comm_chan_rx.try_receive() == Err(mpsc::TryRecvError::Empty));
        let _ = ping_sender.halt();
    }

    #[test]
    fn calling_start_after_halt_is_ignored() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (comm_chan_tx, comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_sender = PingSender::new(icmpv4, socket_mock, comm_chan_tx);

        let _ = ping_sender.halt();

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(2, [ip_127_0_0_1].into());

        assert!(comm_chan_rx.try_receive() == Err(mpsc::TryRecvError::Empty));
        assert!(vec![State::New, State::Halted] == ping_sender.get_states());
    }

    #[test]
    fn calling_start_a_second_time_is_ignored() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (comm_chan_tx, comm_chan_rx) = crate::channel::create_sync_channel(CHANNEL_SIZE);

        let mut ping_sender = PingSender::new(icmpv4, socket_mock, comm_chan_tx);

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        let ip_255_255_255_255 = Ipv4Addr::new(255, 255, 255, 255);
        ping_sender.start(1, [ip_127_0_0_1].into());
        ping_sender.start(1, [ip_255_255_255_255].into());

        let send_record = comm_chan_rx.receive();
        let _ = ping_sender.halt();

        assert!(send_record.is_ok());
        let (_payload_size, ip_addr, _sequence_number, _time) = send_record.unwrap();
        assert!(ip_127_0_0_1 == ip_addr);
        assert!(comm_chan_rx.try_receive() == Err(mpsc::TryRecvError::Empty));
    }
}
