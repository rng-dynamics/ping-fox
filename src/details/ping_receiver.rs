use crate::details::icmp::v4::IcmpV4;
use crate::details::icmp::v4::TSocket;
use crate::details::ping_data_buffer::PingDataBuffer;
use crate::details::records::PingReceiveRecord;
use crate::details::PingResult;
use crate::PingReceive;
use crate::PingSentToken;
use std::sync::Arc;

pub(crate) struct PingReceiver<S> {
    icmpv4: Arc<IcmpV4<S>>,
    ping_data_buffer: PingDataBuffer,
}

impl<S> PingReceiver<S>
where
    S: TSocket + 'static,
{
    pub(crate) fn new(icmpv4: Arc<IcmpV4<S>>, ping_data_buffer: PingDataBuffer) -> Self {
        PingReceiver { icmpv4, ping_data_buffer }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn receive_aux(&self, token: PingSentToken) -> PingResult<PingReceiveRecord> {
        let _ = token;
        // (2) Receive on socket.
        let recv_echo_result = self.icmpv4.try_receive();
        match recv_echo_result {
            Ok(None) => {
                // Timeout: nothing received.
                Ok(PingReceiveRecord::Timeout)
            }
            Err(e) => Err(e.into()),
            Ok(Some(ping_receive_data)) => {
                tracing::trace!("icmpv4 received");
                // (3) Send ping-received-record.
                Ok(PingReceiveRecord::Data(ping_receive_data))
            }
        }
    }

    pub(crate) fn receive(&mut self, token: PingSentToken) -> PingResult<PingReceive> {
        match self.receive_aux(token) {
            Err(e) => Err(e),
            Ok(PingReceiveRecord::Timeout) => Ok(PingReceive::Timeout),
            Ok(PingReceiveRecord::Data(data)) => {
                let _ = self.ping_data_buffer.process_send_records();
                let output = self.ping_data_buffer.process_receive_record(&data)?;
                Ok(PingReceive::Data(output))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::details::icmp::v4::tests::OnReceive;
    use crate::details::icmp::v4::tests::OnSend;
    use crate::details::icmp::v4::tests::SocketMock;
    use crate::details::records::ping_send_record_channel;
    use crate::details::records::PingReceiveRecord;

    #[test]
    fn receive_ping_packages_success() {
        let socket = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnDefault(2));
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (_tx, rx) = ping_send_record_channel(1);
        let ping_data_buffer = PingDataBuffer::new(rx);
        let ping_receiver = PingReceiver::new(icmpv4, ping_data_buffer);

        let recv_record_1 = ping_receiver.receive_aux(PingSentToken {}).unwrap();
        let recv_record_2 = ping_receiver.receive_aux(PingSentToken {}).unwrap();
        let recv_record_3 = ping_receiver.receive_aux(PingSentToken {}).unwrap();

        assert!(matches!(recv_record_1, PingReceiveRecord::Data(_)));
        assert!(matches!(recv_record_2, PingReceiveRecord::Data(_)));
        assert!(matches!(recv_record_3, PingReceiveRecord::Timeout));
    }

    #[test]
    fn when_socket_fails_then_ping_receiver_returns_timeout() {
        let socket = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnWouldBlock);
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (_tx, rx) = ping_send_record_channel(1);
        let ping_data_buffer = PingDataBuffer::new(rx);
        let ping_receiver = PingReceiver::new(icmpv4, ping_data_buffer);

        let recv_record = ping_receiver.receive_aux(PingSentToken {}).unwrap();

        assert!(matches!(recv_record, PingReceiveRecord::Timeout));
    }
}
