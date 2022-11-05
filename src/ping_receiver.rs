use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

// use crate::p_set::*;
use crate::ping_send_sync_event_channel;
use crate::FinalPingDataT;
use crate::IcmpV4;
use crate::PingError;

use crate::event::*;
use crate::ping_sent_sync_event::*;

use crate::PingResult;

pub(crate) struct PingReceiver<S> {
    states: Vec<State>,
    icmpv4: Arc<IcmpV4>,
    socket: Arc<S>,

    ping_sent_sync_event_rx: Option<PingSentSyncEventReceiver>,
    ping_received_event_tx: PingReceiveEventSender,

    halt_tx: mpsc::Sender<()>,
    halt_rx: Option<mpsc::Receiver<()>>,
    thread_handle: Option<JoinHandle<()>>,
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
        // chan_rx: crate::Receiver,
        ping_sent_sync_event_rx: PingSentSyncEventReceiver,
        ping_received_event_tx: PingReceiveEventSender,

        channel_size: usize,
    ) -> Self {
        let (halt_tx, halt_rx) = mpsc::channel::<()>();
        PingReceiver {
            states: vec![State::New],
            icmpv4,
            socket,

            ping_sent_sync_event_rx: Some(ping_sent_sync_event_rx),
            ping_received_event_tx,

            halt_tx,
            halt_rx: Some(halt_rx),
            thread_handle: None,
        }
    }

    pub(crate) fn get_states(&self) -> Vec<State> {
        self.states.clone()
    }

    // pub(crate) fn __next_result
    // pub fn next_result(&self) -> PingResult<FinalPingDataT> {
    //     if *self.states.last().expect("logic error") == State::Halted {
    //         return Err(PingError {
    //             message: "cannot get next result when PingReceiver is halted".to_string(),
    //             source: None,
    //         }
    //         .into());
    //     }

    //     match self.results_rx.try_recv() {
    //         Err(e) => Err(e.into()),
    //         Ok(Err(e)) => Err(e),
    //         Ok(Ok(ok)) => Ok(ok),
    //     }
    // }

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

        let icmpv4 = self.icmpv4.clone();
        let socket = self.socket.clone();

        let ping_sent_sync_event_rx = self.ping_sent_sync_event_rx.take().expect("logic error");
        let ping_received_event_tx = self.ping_received_event_tx.clone();

        let halt_rx = self.halt_rx.take().expect("logic error");

        self.thread_handle = Some(std::thread::spawn(move || {
            'outer: loop {
                // (1) Wait for sync-event from PingSender.
                let ping_sent_sync_event_recv = ping_sent_sync_event_rx.recv();
                if let Err(_) = ping_sent_sync_event_recv {
                    println!("log INFO: mpsc::Receiver::recv() failed");
                    break 'outer;
                }
                println!("log INFO: mpsc::Receiver::recv() success");

                // TODO: set timeout to rather big (config) value.
                // (2) Receive on socket.
                let recv_echo_result = icmpv4.try_receive(&*socket);
                match recv_echo_result {
                    Ok(None) => {
                        // Timeout: nothing received.
                        println!("log TRACE: try_receive Ok(None)");
                        ping_received_event_tx.send(PingReceiveEvent::Timeout);
                    }
                    Err(e) => {
                        println!("log TRACE: try_receive Err(e)");
                        println!("log ERROR: error receiving icmp: {}", e);
                    }
                    Ok(Some((packet_size, ip_addr, sequence_number, receive_time))) => {
                        println!("log TRACE: try_receive Ok(Some((ip, sn)))");
                        println!("log TRACE: icmpv4 received");
                        // (3) Send ping-received-event.
                        ping_received_event_tx.send(PingReceiveEvent::Data(PingReceiveEventData {
                            packet_size,
                            ip_addr,
                            sequence_number,
                            receive_time,
                        }));
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
    use crate::event::ping_receive_event_channel;
    use crate::event::ping_send_event_channel;
    use crate::ping_sender::PingSender;
    use crate::socket::tests::OnReceive;
    use crate::socket::tests::OnSend;
    use crate::socket::tests::SocketMock;
    // use crate::

    use std::net::Ipv4Addr;

    const CHANNEL_SIZE: usize = 8;

    #[test]
    fn entity_states() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (tx_1, rx_1) = ping_send_sync_event_channel();
        let (tx_2, rx_2) = ping_receive_event_channel();

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, rx_1, tx_2, CHANNEL_SIZE);

        assert!(vec![State::New] == ping_receiver.get_states());
        ping_receiver.start();
        assert!(vec![State::New, State::Receiving] == ping_receiver.get_states());
        drop(tx_1);
        drop(rx_2);
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
        let (tx_1, rx_1) = ping_send_sync_event_channel();
        let (tx_2, rx_2) = ping_receive_event_channel();

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, rx_1, tx_2, CHANNEL_SIZE);
        ping_receiver.start();

        drop(ping_receiver);
    }

    #[test]
    fn receive_ping_packets_success_1() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(2),
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (tx_1, rx_1) = ping_send_sync_event_channel();
        let (tx_2, rx_2) = ping_receive_event_channel();

        tx_1.send(PingSentSyncEvent).unwrap();
        tx_1.send(PingSentSyncEvent).unwrap();

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, rx_1, tx_2, CHANNEL_SIZE);
        ping_receiver.start();

        println!("log TRACE: receive_ping_packets_success: will call next_result");
        let ping_receiver_result_1 = rx_2.recv();
        let ping_receiver_result_2 = rx_2.recv();
        println!("log TRACE: receive_ping_packets_success: call next_result done");

        drop(tx_1);
        // drop(rx_2);
        let _ = ping_receiver.halt();

        // assert!(next_ping_receiver_result.is_ok());
    }

    #[test]
    fn receive_ping_packets_success_2() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(2),
        ));
        let icmpv4 = Arc::new(IcmpV4::create());

        let (tx_1, rx_1) = ping_send_sync_event_channel();
        let (tx_2, rx_2) = ping_receive_event_channel();
        let (ping_send_event_tx, ping_send_event_rx) = ping_send_event_channel();

        let mut ping_sender = PingSender::new(
            icmpv4.clone(),
            socket_mock.clone(),
            ping_send_event_tx,
            tx_1,
        );

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(3, [ip_127_0_0_1].into());

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, rx_1, tx_2, CHANNEL_SIZE);
        ping_receiver.start();

        println!("log TRACE: receive_ping_packets_success: will call next_result");
        let ping_receiver_result_1 = rx_2.recv();
        let ping_receiver_result_2 = rx_2.recv();
        let ping_receiver_result_3 = rx_2.recv();
        println!("log TRACE: receive_ping_packets_success: call next_result done");

        let _ = ping_sender.halt();
        println!("log TRACE: PingSender.halt() done");
        drop(ping_sender);
        let _ = ping_receiver.halt();
        println!("log TRACE: PingReceiver.halt() done");

        // assert!(PingReceiveEvent::Data(PingReceiveEventData{packet_size, ip_addr, sequence_number}) == ping_receiver_result_1.unwrap());
        assert!(matches!(
            ping_receiver_result_1.unwrap(),
            PingReceiveEvent::Data(..)
        ));
        // assert!(PingReceiveEvent::Data(PingReceiveEventData{packet_size, ip_addr, sequence_number}) == ping_receiver_result_2.unwrap());
        assert!(matches!(
            ping_receiver_result_2.unwrap(),
            PingReceiveEvent::Data(..)
        ));
        assert!(PingReceiveEvent::Timeout == ping_receiver_result_3.unwrap());
    }

    #[test]
    fn when_socket_fails_then_ping_receiver_returns_timeout() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());

        let (tx_1, rx_1) = ping_send_sync_event_channel();
        let (tx_2, rx_2) = ping_receive_event_channel();
        let (ping_sent_event_tx, ping_sent_event_rx) = ping_send_event_channel();

        let mut ping_sender = PingSender::new(
            icmpv4.clone(),
            socket_mock.clone(),
            ping_sent_event_tx,
            tx_1,
        );

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(1, [ip_127_0_0_1].into());

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, rx_1, tx_2, CHANNEL_SIZE);
        ping_receiver.start();

        println!("log TRACE: receive_ping_packets_success: will call next_result");
        let ping_receiver_result_1 = rx_2.recv();
        println!("log TRACE: receive_ping_packets_success: call next_result done");

        let _ = ping_sender.halt();
        println!("log TRACE: PingSender.halt() done");
        drop(ping_sender);
        let _ = ping_receiver.halt();
        println!("log TRACE: PingReceiver.halt() done");

        assert!(PingReceiveEvent::Timeout == ping_receiver_result_1.unwrap());
    }

    #[test]
    fn calling_start_after_halt_is_ignored() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(1),
        ));
        let icmpv4 = Arc::new(IcmpV4::create());

        let (tx_1, rx_1) = ping_send_sync_event_channel();
        let (tx_2, rx_2) = ping_receive_event_channel();
        let (ping_sent_event_tx, ping_sent_event_rx) = ping_send_event_channel();

        let mut ping_sender = PingSender::new(
            icmpv4.clone(),
            socket_mock.clone(),
            ping_sent_event_tx,
            tx_1,
        );

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.start(1, [ip_127_0_0_1].into());

        let mut ping_receiver = PingReceiver::new(icmpv4, socket_mock, rx_1, tx_2, CHANNEL_SIZE);

        let _ = ping_sender.halt();
        println!("log TRACE: PingSender.halt() done");
        drop(ping_sender);
        let _ = ping_receiver.halt();
        println!("log TRACE: PingReceiver.halt() done");

        ping_receiver.start();
        println!("log TRACE: PingReceiver.start() done");

        let ping_receiver_result_1 = rx_2.try_recv();
        println!("log TRACE: rx_2.recv() done");
        assert!(Err(mpsc::TryRecvError::Empty) == ping_receiver_result_1);
        assert!(vec![State::New, State::Halted] == ping_receiver.get_states());
    }

    // #[test]
    // fn calling_start_a_second_time_is_ignored() {
    // }
}
