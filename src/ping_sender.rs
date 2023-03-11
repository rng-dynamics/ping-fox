use crate::event::{PingSendEvent, PingSendEventSender};
use crate::icmp::v4::SequenceNumber;
use crate::IcmpV4;
use crate::PingResult;
use crate::PingSentToken;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::Arc;

pub struct PingSender<S> {
    icmpv4: Arc<IcmpV4<S>>,
    ping_sent_event_tx: PingSendEventSender,
    ips: VecDeque<Ipv4Addr>,
    sequence_number: SequenceNumber,
}

impl<S> PingSender<S>
where
    S: crate::icmp::v4::Socket + 'static,
{
    pub(crate) fn new(
        icmpv4: Arc<IcmpV4<S>>,
        ping_sent_event_tx: PingSendEventSender,
        ips: VecDeque<Ipv4Addr>,
    ) -> Self {
        PingSender {
            icmpv4,
            ping_sent_event_tx,
            ips,
            sequence_number: SequenceNumber::start_value2(),
        }
    }

    fn send_one(&self, ip: Ipv4Addr, sequence_number: SequenceNumber) -> PingResult<()> {
        // (1) Send ping.
        let (payload_size, ip_addr, sequence_number, send_time) =
            self.icmpv4.send_one_ping(ip, sequence_number)?;
        tracing::trace!("icmpv4 sent");

        // (2) Dispatch data to PingDataBuffer
        self.ping_sent_event_tx.send(PingSendEvent {
            payload_size,
            ip_addr,
            sequence_number,
            send_time,
        })?;
        Ok(())
    }

    pub fn send_ping_to_each_address(&mut self) -> PingResult<Vec<PingSentToken>> {
        let mut result = Vec::with_capacity(self.ips.len());
        let sequence_number = self.sequence_number;
        self.sequence_number = self.sequence_number.next();
        for ip in &self.ips {
            // (2) Send ping
            let send_one_result = self.send_one(*ip, sequence_number);
            match send_one_result {
                Err(e) => {
                    return Err(e);
                }
                Ok(()) => {
                    result.push(PingSentToken {});
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ping_send_event_channel;
    use crate::icmp::v4::tests::OnReceive;
    use crate::icmp::v4::tests::OnSend;
    use crate::icmp::v4::tests::SocketMock;
    use std::sync::mpsc;

    #[test]
    fn send_ping_packages_success() {
        let socket = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnDefault(2));
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (tx, rx) = ping_send_event_channel(2);
        let ping_sender = PingSender::new(icmpv4, tx, [].into());

        let localhost = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.send_one(localhost, SequenceNumber(0)).unwrap();
        ping_sender.send_one(localhost, SequenceNumber(1)).unwrap();

        let ping_sent_event_1 = rx.recv();
        let ping_sent_event_2 = rx.recv();

        assert!(ping_sent_event_1.is_ok());
        let PingSendEvent {
            payload_size: _,
            ip_addr,
            sequence_number,
            send_time: _,
        } = ping_sent_event_1.unwrap();
        assert!(localhost == ip_addr);
        assert!(sequence_number == SequenceNumber(0));

        assert!(ping_sent_event_2.is_ok());
        let PingSendEvent {
            payload_size: _,
            ip_addr,
            sequence_number,
            send_time: _,
        } = ping_sent_event_2.unwrap();
        assert!(localhost == ip_addr);
        assert!(sequence_number == SequenceNumber(1));
    }

    #[test]
    fn when_socket_fails_then_ping_sender_fails() {
        let socket = SocketMock::new(OnSend::ReturnErr, OnReceive::ReturnWouldBlock);
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (tx, rx) = ping_send_event_channel(1);
        let ping_sender = PingSender::new(icmpv4, tx, [].into());

        let localhost = Ipv4Addr::new(127, 0, 0, 1);
        let send_result = ping_sender.send_one(localhost, SequenceNumber(0));

        assert!(send_result.is_err());
        assert!(rx.try_recv() == Err(mpsc::TryRecvError::Empty));
    }
}
