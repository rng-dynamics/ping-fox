use crate::event::{PingReceiveEvent, PingReceiveEventSender};
use crate::IcmpV4;
use crate::PingResult;
use std::sync::Arc;

pub(crate) struct PingReceiver<S> {
    socket: Arc<S>,
    ping_received_event_tx: PingReceiveEventSender,
}

impl<S> PingReceiver<S>
where
    S: crate::icmp::v4::socket::Socket + 'static,
{
    pub(crate) fn new(socket: Arc<S>, ping_received_event_tx: PingReceiveEventSender) -> Self {
        PingReceiver {
            socket,
            ping_received_event_tx,
        }
    }

    pub(crate) fn receive(&self) -> PingResult<()> {
        // (2) Receive on socket.
        let recv_echo_result = IcmpV4::try_receive(&*self.socket);
        match recv_echo_result {
            Ok(None) => {
                // Timeout: nothing received.
                self.ping_received_event_tx
                    .send(PingReceiveEvent::Timeout)?;
            }
            Err(e) => {
                tracing::error!("error receiving icmp: {}", e);
            }
            Ok(Some(ping_receive_data)) => {
                tracing::trace!("icmpv4 received");
                // (3) Send ping-received-event.
                self.ping_received_event_tx
                    .send(PingReceiveEvent::Data(ping_receive_data))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{ping_receive_event_channel, ping_send_event_channel};
    use crate::icmp::v4::socket::tests::OnReceive;
    use crate::icmp::v4::socket::tests::OnSend;
    use crate::icmp::v4::socket::tests::SocketMock;
    use crate::PingSender;
    use std::net::Ipv4Addr;

    const CHANNEL_SIZE: usize = 8;

    #[test]
    fn receive_ping_packages_success_1() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(2),
        ));
        let (tx_2, rx_2) = ping_receive_event_channel(CHANNEL_SIZE);

        let ping_receiver = PingReceiver::new(socket_mock, tx_2);
        ping_receiver.receive().unwrap();
        ping_receiver.receive().unwrap();

        let ping_receive_event_1 = rx_2.recv();
        let ping_receive_event_2 = rx_2.recv();

        assert!(ping_receive_event_1.is_ok());
        assert!(ping_receive_event_2.is_ok());
    }

    #[test]
    fn receive_ping_packages_success_2() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnDefault(2),
        ));
        let icmpv4 = Arc::new(IcmpV4::create());

        let (tx_2, rx_2) = ping_receive_event_channel(CHANNEL_SIZE);
        let (ping_send_event_tx, _ping_send_event_rx) = ping_send_event_channel(CHANNEL_SIZE);

        let ping_sender = PingSender::new(icmpv4, socket_mock.clone(), ping_send_event_tx);

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.send_one(ip_127_0_0_1, 0).unwrap();
        ping_sender.send_one(ip_127_0_0_1, 1).unwrap();
        ping_sender.send_one(ip_127_0_0_1, 2).unwrap();

        let ping_receiver = PingReceiver::new(socket_mock, tx_2);
        ping_receiver.receive().unwrap();
        ping_receiver.receive().unwrap();
        ping_receiver.receive().unwrap();

        let ping_receiver_result_1 = rx_2.recv();
        let ping_receiver_result_2 = rx_2.recv();
        let ping_receiver_result_3 = rx_2.recv();

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
        let (tx_2, rx_2) = ping_receive_event_channel(CHANNEL_SIZE);
        let (ping_sent_event_tx, _ping_sent_event_rx) = ping_send_event_channel(CHANNEL_SIZE);
        let ping_sender = PingSender::new(icmpv4, socket_mock.clone(), ping_sent_event_tx);
        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.send_one(ip_127_0_0_1, 0).unwrap();

        let ping_receiver = PingReceiver::new(socket_mock, tx_2);
        let receive_result = ping_receiver.receive();

        assert!(receive_result.is_ok());
        assert!(rx_2.recv().unwrap() == PingReceiveEvent::Timeout);
    }
}
