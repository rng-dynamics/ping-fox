use std::sync::Arc;

use crate::event::*;
use crate::IcmpV4;
use crate::PingResult;

pub(crate) struct PingReceiver<S> {
    icmpv4: Arc<IcmpV4>,
    socket: Arc<S>,
    ping_received_event_tx: PingReceiveEventSender,
}

impl<S> PingReceiver<S>
where
    S: crate::Socket + 'static,
{
    pub(crate) fn new(
        icmpv4: Arc<IcmpV4>,
        socket: Arc<S>,
        ping_received_event_tx: PingReceiveEventSender,
    ) -> Self {
        PingReceiver {
            icmpv4,
            socket,
            ping_received_event_tx,
        }
    }

    pub(crate) fn receive(&self) -> PingResult<()> {
        // (2) Receive on socket.
        let recv_echo_result = self.icmpv4.try_receive(&*self.socket);
        match recv_echo_result {
            Ok(None) => {
                // Timeout: nothing received.
                println!("log TRACE: try_receive Ok(None)");
                self.ping_received_event_tx
                    .send(PingReceiveEvent::Timeout)?;
            }
            Err(e) => {
                println!("log TRACE: try_receive Err(e)");
                println!("log ERROR: error receiving icmp: {}", e);
            }
            Ok(Some((packet_size, ip_addr, sequence_number, receive_time))) => {
                println!("log TRACE: try_receive Ok(Some((ip, sn)))");
                println!("log TRACE: icmpv4 received");
                // (3) Send ping-received-event.
                self.ping_received_event_tx
                    .send(PingReceiveEvent::Data(PingReceiveEventData {
                        packet_size,
                        ip_addr,
                        sequence_number,
                        receive_time,
                    }))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PingSender;
    use crate::socket::tests::OnReceive;
    use crate::socket::tests::OnSend;
    use crate::socket::tests::SocketMock;
    use std::net::Ipv4Addr;

    #[test]
    fn receive_ping_packets_success_1() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(2),
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (tx_2, rx_2) = ping_receive_event_channel();

        let ping_receiver = PingReceiver::new(icmpv4, socket_mock, tx_2);
        ping_receiver.receive().unwrap();
        ping_receiver.receive().unwrap();

        let ping_receive_event_1 = rx_2.recv();
        let ping_receive_event_2 = rx_2.recv();

        assert!(ping_receive_event_1.is_ok());
        assert!(ping_receive_event_2.is_ok());
    }

    #[test]
    fn receive_ping_packets_success_2() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(2),
        ));
        let icmpv4 = Arc::new(IcmpV4::create());

        let (tx_2, rx_2) = ping_receive_event_channel();
        let (ping_send_event_tx, _ping_send_event_rx) = ping_send_event_channel();

        let ping_sender = PingSender::new(icmpv4.clone(), socket_mock.clone(), ping_send_event_tx);

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.send_one(ip_127_0_0_1, 0).unwrap();
        ping_sender.send_one(ip_127_0_0_1, 1).unwrap();
        ping_sender.send_one(ip_127_0_0_1, 2).unwrap();

        let ping_receiver = PingReceiver::new(icmpv4, socket_mock, tx_2);
        ping_receiver.receive().unwrap();
        ping_receiver.receive().unwrap();
        ping_receiver.receive().unwrap();

        println!("log TRACE: receive_ping_packets_success: will call next_result");
        let ping_receiver_result_1 = rx_2.recv();
        let ping_receiver_result_2 = rx_2.recv();
        let ping_receiver_result_3 = rx_2.recv();
        println!("log TRACE: receive_ping_packets_success: call next_result done");

        assert!(matches!(
            ping_receiver_result_1.unwrap(),
            PingReceiveEvent::Data(..)
        ));
        assert!(matches!(
            ping_receiver_result_2.unwrap(),
            PingReceiveEvent::Data(..)
        ));
        assert!(ping_receiver_result_3.unwrap() == PingReceiveEvent::Timeout);
    }

    #[test]
    fn when_socket_fails_then_ping_receiver_returns_timeout() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (tx_2, rx_2) = ping_receive_event_channel();
        let (ping_sent_event_tx, _ping_sent_event_rx) = ping_send_event_channel();
        let ping_sender = PingSender::new(icmpv4.clone(), socket_mock.clone(), ping_sent_event_tx);
        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.send_one(ip_127_0_0_1, 0).unwrap();

        let ping_receiver = PingReceiver::new(icmpv4, socket_mock, tx_2);
        let receive_result = ping_receiver.receive();

        assert!(receive_result.is_ok());
        assert!(rx_2.recv().unwrap() == PingReceiveEvent::Timeout);
    }
}
