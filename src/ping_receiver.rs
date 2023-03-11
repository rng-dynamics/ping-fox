use crate::event::PingReceiveEvent;
use crate::ping_data_buffer::PingDataBuffer;
use crate::IcmpV4;
use crate::PingReceiveResult;
use crate::PingResult;
use crate::PingSentToken;
use std::sync::Arc;

pub struct PingReceiver<S> {
    icmpv4: Arc<IcmpV4<S>>,
    ping_data_buffer: PingDataBuffer,
}

impl<S> PingReceiver<S>
where
    S: crate::icmp::v4::Socket + 'static,
{
    pub(crate) fn new(icmpv4: Arc<IcmpV4<S>>, ping_data_buffer: PingDataBuffer) -> Self {
        PingReceiver {
            icmpv4,
            ping_data_buffer,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn receive(&self, token: PingSentToken) -> PingResult<PingReceiveEvent> {
        let _ = token;
        // (2) Receive on socket.
        let recv_echo_result = self.icmpv4.try_receive();
        match recv_echo_result {
            Ok(None) => {
                // Timeout: nothing received.
                Ok(PingReceiveEvent::Timeout)
            }
            Err(e) => Err(e.into()),
            Ok(Some(ping_receive_data)) => {
                tracing::trace!("icmpv4 received");
                // (3) Send ping-received-event.
                Ok(PingReceiveEvent::Data(ping_receive_data))
            }
        }
    }

    pub fn receive_ping(&mut self, token: PingSentToken) -> PingResult<PingReceiveResult> {
        match self.receive(token) {
            Err(e) => Err(e),
            Ok(PingReceiveEvent::Timeout) => {
                // TODO
                Ok(PingReceiveResult::Timeout)
            }
            Ok(PingReceiveEvent::Data(data)) => {
                let _ = self.ping_data_buffer.process_send_events();
                let output = self.ping_data_buffer.process_receive_event(&data)?;
                Ok(PingReceiveResult::Data(output))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ping_send_event_channel;
    use crate::event::PingReceiveEvent;
    use crate::icmp::v4::tests::OnReceive;
    use crate::icmp::v4::tests::OnSend;
    use crate::icmp::v4::tests::SocketMock;

    #[test]
    fn receive_ping_packages_success() {
        let socket = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnDefault(2));
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (_tx, rx) = ping_send_event_channel(1);
        let ping_data_buffer = PingDataBuffer::new(rx);
        let ping_receiver = PingReceiver::new(icmpv4, ping_data_buffer);

        let recv_event_1 = ping_receiver.receive(PingSentToken {}).unwrap();
        let recv_event_2 = ping_receiver.receive(PingSentToken {}).unwrap();
        let recv_event_3 = ping_receiver.receive(PingSentToken {}).unwrap();

        assert!(matches!(recv_event_1, PingReceiveEvent::Data(_)));
        assert!(matches!(recv_event_2, PingReceiveEvent::Data(_)));
        assert!(matches!(recv_event_3, PingReceiveEvent::Timeout));
    }

    #[test]
    fn when_socket_fails_then_ping_receiver_returns_timeout() {
        let socket = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnWouldBlock);
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (_tx, rx) = ping_send_event_channel(1);
        let ping_data_buffer = PingDataBuffer::new(rx);
        let ping_receiver = PingReceiver::new(icmpv4, ping_data_buffer);

        let recv_event = ping_receiver.receive(PingSentToken {}).unwrap();

        assert!(matches!(recv_event, PingReceiveEvent::Timeout));
    }
}
