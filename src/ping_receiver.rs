use crate::event::PingReceiveEvent;
use crate::ping_data_buffer::PingDataBuffer;
use crate::ping_runner_v2::PingResult;
use crate::ping_runner_v2::PingSentEvidence;
use crate::{IcmpV4, PingOutputData};
use std::sync::Arc;

pub struct PingReceiver<S> {
    icmpv4: Arc<IcmpV4<S>>,
    ping_data_buffer: PingDataBuffer,
}

impl<S> PingReceiver<S>
where
    S: crate::icmp::v4::Socket + 'static,
{
    // TODO: rename to new
    pub(crate) fn new(icmpv4: Arc<IcmpV4<S>>, ping_data_buffer: PingDataBuffer) -> Self {
        PingReceiver {
            icmpv4,
            ping_data_buffer: ping_data_buffer.into(),
        }
    }

    // TODO: make private
    #[allow(clippy::needless_pass_by_value)] // TODO: keep ?
    pub(crate) fn receive(&self, evidence: PingSentEvidence) -> PingResult<PingReceiveEvent> {
        let _ = evidence;
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

    pub fn receive_ping(
        &mut self,
        evidence: PingSentEvidence,
    ) -> PingResult<Option<PingOutputData>> {
        match self.receive(evidence) {
            Err(e) => Err(e),
            Ok(PingReceiveEvent::Timeout) => {
                // TODO
                Ok(None)
            }
            Ok(PingReceiveEvent::Data(data)) => {
                let _ = self.ping_data_buffer.process_send_events();
                let output = self.ping_data_buffer.process_receive_event2(&data)?;
                Ok(Some(output))
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

        let recv_event_1 = ping_receiver.receive(PingSentEvidence {}).unwrap();
        let recv_event_2 = ping_receiver.receive(PingSentEvidence {}).unwrap();
        let recv_event_3 = ping_receiver.receive(PingSentEvidence {}).unwrap();

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

        let recv_event = ping_receiver.receive(PingSentEvidence {}).unwrap();

        assert!(matches!(recv_event, PingReceiveEvent::Timeout));
    }
}
