use super::TSocket;
use crate::details::icmp::v4::SequenceNumber;
use crate::details::records::PingReceiveRecordData;
use crate::details::PingError;
use pnet_packet::icmp::{
    echo_reply::EchoReplyPacket,
    echo_request::{EchoRequestPacket as EchoRequestPacketV4, MutableEchoRequestPacket as MutableEchoRequestPacketV4},
    IcmpPacket, IcmpTypes,
};
use pnet_packet::Packet;
use rand::Rng;
use std::io;
use std::net::{IpAddr, Ipv4Addr};
use std::result::Result;
use std::time::Instant;

const PAYLOAD_SIZE: usize = 56;

pub(crate) struct IcmpV4<S> {
    payload: [u8; PAYLOAD_SIZE],
    socket: S,
}

impl<S> IcmpV4<S>
where
    S: TSocket + 'static,
{
    pub(crate) fn new(socket: S) -> IcmpV4<S> {
        let mut payload = [0u8; PAYLOAD_SIZE];
        rand::thread_rng().fill(&mut payload[..]);
        IcmpV4 { payload, socket }
    }

    pub(crate) fn send_to(
        &self,
        ipv4: Ipv4Addr,
        sequence_number: SequenceNumber,
    ) -> Result<(usize, IpAddr, SequenceNumber, Instant), PingError> {
        let ip_addr = IpAddr::V4(ipv4);
        let addr = std::net::SocketAddr::new(ip_addr, 0);

        let package = new_icmpv4_package(sequence_number, &self.payload)
            .ok_or(PingError { message: "could not create ICMP package".to_owned() })?;

        let packet = pnet_packet::Packet::packet(&package);
        let addr2: socket2::SockAddr = addr.into();
        let start_time: Instant = Instant::now();
        self.socket.send_to(packet, &addr2)?;

        Ok((PAYLOAD_SIZE, ip_addr, sequence_number, start_time))
    }

    pub(crate) fn try_receive(&self) -> std::result::Result<Option<PingReceiveRecordData>, io::Error> {
        let mut buf1 = [0u8; 128];
        match self.socket.recv_from(&mut buf1) {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
            Ok((package_size, ip_addr, ttl)) => {
                let receive_time: Instant = Instant::now();
                let echo_reply_package = EchoReplyPacket::new(&buf1).expect("could not initialize echo reply package");
                let sequence_number: SequenceNumber = echo_reply_package.get_sequence_number().into();
                Ok(Some(PingReceiveRecordData {
                    package_size,
                    ip_addr,
                    ttl,
                    sequence_number,
                    receive_time,
                }))
            }
        }
    }
}

pub(crate) fn new_icmpv4_package(
    sequence_number: SequenceNumber,
    payload: &[u8],
) -> Option<MutableEchoRequestPacketV4<'static>> {
    let buf = vec![0u8; EchoRequestPacketV4::minimum_packet_size() + payload.len()];
    let mut package = MutableEchoRequestPacketV4::owned(buf)?;
    package.set_sequence_number(sequence_number.into());
    package.set_identifier(0);
    package.set_icmp_type(IcmpTypes::EchoRequest);
    package.set_payload(payload);

    package.set_checksum(0_u16);
    let checksum = pnet_packet::icmp::checksum(&IcmpPacket::new(package.packet())?);
    package.set_checksum(checksum);
    Some(package)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::details::icmp::v4::tests::OnReceive;
    use crate::details::icmp::v4::tests::OnSend;
    use crate::details::icmp::v4::tests::SocketMock;

    #[test]
    fn test_send_one_ping() {
        let socket_mock = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnWouldBlock);
        let socket_mock_clone = socket_mock.clone();
        let icmpv4 = IcmpV4::new(socket_mock_clone);

        let addr = Ipv4Addr::new(127, 0, 0, 1);
        let sequence_number = SequenceNumber::start_value();
        let result = icmpv4.send_to(addr, sequence_number);

        assert!(result.is_ok());
        socket_mock
            .should_send_number_of_messages(1)
            .should_send_to_address(&IpAddr::V4(addr));
    }

    #[test]
    fn test_try_receive() {
        let socket_mock: SocketMock = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnDefault(1));
        let socket_mock_clone = socket_mock.clone();
        let icmpv4 = IcmpV4::new(socket_mock_clone);

        let result = icmpv4.try_receive();

        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        let PingReceiveRecordData { package_size, ip_addr, ttl: _, sequence_number: _, receive_time: _ } =
            result.unwrap().unwrap();
        assert!(package_size >= EchoReplyPacket::minimum_packet_size());
        assert!(ip_addr == Ipv4Addr::new(127, 0, 0, 1));
        socket_mock.should_receive_number_of_messages(1);
    }
}
