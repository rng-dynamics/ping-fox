use crate::details::icmp::v4::IcmpV4;
use crate::details::icmp::v4::SequenceNumber;
use crate::details::icmp::v4::TSocket;
use crate::details::records::{PingSendRecord, PingSendRecordSender};
use crate::details::PingResult;
use crate::PingSentToken;
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::Arc;

pub(crate) struct PingSender<S> {
    icmpv4: Arc<IcmpV4<S>>,
    ping_sent_record_tx: PingSendRecordSender,
    sequence_numbers: HashMap<IpAddr, SequenceNumber>,
}

impl<S> PingSender<S>
where
    S: TSocket + 'static,
{
    pub(crate) fn new(icmpv4: Arc<IcmpV4<S>>, ping_sent_record_tx: PingSendRecordSender) -> Self {
        PingSender { icmpv4, ping_sent_record_tx, sequence_numbers: HashMap::new() }
    }

    fn send_to_details(&self, ip: Ipv4Addr, sequence_number: SequenceNumber) -> PingResult<()> {
        // (1) Send ping.
        let (payload_size, ip_addr, sequence_number, send_time) = self.icmpv4.send_to(ip, sequence_number)?;
        tracing::trace!("icmpv4 sent");

        // (2) Dispatch data to PingDataBuffer
        self.ping_sent_record_tx
            .send(PingSendRecord { payload_size, ip_addr, sequence_number, send_time })?;
        Ok(())
    }

    pub(crate) fn send_to(&mut self, ip: Ipv4Addr) -> PingResult<PingSentToken> {
        let sequence_number = match self.sequence_numbers.get(&IpAddr::V4(ip)) {
            Some(sequence_number) => sequence_number.next(),
            None => SequenceNumber::start_value(),
        };
        self.sequence_numbers.insert(IpAddr::V4(ip), sequence_number);

        self.send_to_details(ip, sequence_number)?;
        Ok(PingSentToken {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::details::icmp::v4::tests::OnReceive;
    use crate::details::icmp::v4::tests::OnSend;
    use crate::details::icmp::v4::tests::SocketMock;
    use crate::details::records::ping_send_record_channel;
    use std::sync::mpsc;

    #[test]
    fn send_ping_packages_success() {
        let socket = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnDefault(2));
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (tx, rx) = ping_send_record_channel(2);
        let ping_sender = PingSender::new(icmpv4, tx);

        let localhost = Ipv4Addr::new(127, 0, 0, 1);
        ping_sender.send_to_details(localhost, SequenceNumber::from(1)).unwrap();
        ping_sender.send_to_details(localhost, SequenceNumber::from(2)).unwrap();

        let ping_sent_record_1 = rx.recv();
        let ping_sent_record_2 = rx.recv();

        assert!(ping_sent_record_1.is_ok());
        let PingSendRecord { payload_size: _, ip_addr, sequence_number, send_time: _ } = ping_sent_record_1.unwrap();
        assert!(localhost == ip_addr);
        assert!(sequence_number == SequenceNumber::from(1));

        assert!(ping_sent_record_2.is_ok());
        let PingSendRecord { payload_size: _, ip_addr, sequence_number, send_time: _ } = ping_sent_record_2.unwrap();
        assert!(localhost == ip_addr);
        assert!(sequence_number == SequenceNumber::from(2));
    }

    #[test]
    fn when_socket_fails_then_ping_sender_fails() {
        let socket = SocketMock::new(OnSend::ReturnErr, OnReceive::ReturnWouldBlock);
        let icmpv4 = Arc::new(IcmpV4::new(socket));
        let (tx, rx) = ping_send_record_channel(1);
        let ping_sender = PingSender::new(icmpv4, tx);

        let localhost = Ipv4Addr::new(127, 0, 0, 1);
        let send_result = ping_sender.send_to_details(localhost, SequenceNumber::start_value());

        assert!(send_result.is_err());
        assert!(rx.try_recv() == Err(mpsc::TryRecvError::Empty));
    }
}
