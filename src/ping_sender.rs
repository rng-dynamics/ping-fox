use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::event::{PingSendEvent, PingSendEventSender};
use crate::IcmpV4;
use crate::PingResult;

pub(crate) struct PingSender<S> {
    icmpv4: Arc<IcmpV4>,
    socket: Arc<S>,
    ping_sent_event_tx: PingSendEventSender,
}

impl<S> PingSender<S>
where
    S: crate::Socket + 'static,
{
    pub(crate) fn new(
        icmpv4: Arc<IcmpV4>,
        socket: Arc<S>,
        ping_sent_event_tx: PingSendEventSender,
    ) -> Self {
        PingSender {
            icmpv4,
            socket,
            ping_sent_event_tx,
        }
    }

    pub(crate) fn send_one(&self, ip: Ipv4Addr, sequence_number: u16) -> PingResult<()> {
        // (1) Send ping.
        let (payload_size, ip_addr, sequence_number, send_time) =
            self.icmpv4
                .send_one_ping(&*self.socket, ip, sequence_number)?;

        // (2) Dispatch data to PingDataBuffer
        self.ping_sent_event_tx.send(PingSendEvent {
            payload_size,
            ip_addr,
            sequence_number,
            send_time,
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ping_send_event_channel;
    use crate::socket::tests::OnReceive;
    use crate::socket::tests::OnSend;
    use crate::socket::tests::SocketMock;
    use std::sync::mpsc;

    const CHANNEL_SIZE: usize = 8;

    #[test]
    fn send_ping_packages_success() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnDefault,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (ping_sent_event_tx, ping_sent_event_rx) = ping_send_event_channel(CHANNEL_SIZE);

        let ping_sender = PingSender::new(icmpv4, socket_mock, ping_sent_event_tx);

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.send_one(ip_127_0_0_1, 0).unwrap();
        ping_sender.send_one(ip_127_0_0_1, 1).unwrap();

        let ping_sent_event_1 = ping_sent_event_rx.recv();
        let ping_sent_event_2 = ping_sent_event_rx.recv();

        assert!(ping_sent_event_1.is_ok());
        let PingSendEvent {
            payload_size: _,
            ip_addr,
            sequence_number,
            send_time: _,
        } = ping_sent_event_1.unwrap();
        assert!(ip_127_0_0_1 == ip_addr);
        assert!(sequence_number == 0);

        assert!(ping_sent_event_2.is_ok());
        let PingSendEvent {
            payload_size: _,
            ip_addr,
            sequence_number,
            send_time: _,
        } = ping_sent_event_2.unwrap();
        assert!(ip_127_0_0_1 == ip_addr);
        assert!(sequence_number == 1);
    }

    #[test]
    fn when_socket_fails_then_ping_sender_fails() {
        let socket_mock = Arc::new(SocketMock::new(
            OnSend::ReturnErr,
            OnReceive::ReturnWouldBlock,
        ));
        let icmpv4 = Arc::new(IcmpV4::create());
        let (ping_sent_event_tx, ping_sent_event_rx) = ping_send_event_channel(CHANNEL_SIZE);

        let ping_sender = PingSender::new(icmpv4, socket_mock, ping_sent_event_tx);

        let ip_127_0_0_1 = Ipv4Addr::new(127, 0, 0, 1);
        let send_result = ping_sender.send_one(ip_127_0_0_1, 0);

        assert!(send_result.is_err());
        assert!(ping_sent_event_rx.try_recv() == Err(mpsc::TryRecvError::Empty));
    }
}
