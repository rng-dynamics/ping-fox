use std::io;
use std::net::{IpAddr, Ipv4Addr};
use std::result::Result;
use std::time::Instant;

use pnet_packet::icmp::{
    echo_reply::EchoReplyPacket,
    echo_request::{
        EchoRequestPacket as EchoRequestPacketV4,
        MutableEchoRequestPacket as MutableEchoRequestPacketV4,
    },
    IcmpPacket, IcmpTypes,
};
use pnet_packet::Packet;
use rand::Rng;

use crate::GenericError;
use crate::PingError;

const PAYLOAD_SIZE: usize = 56;

pub struct IcmpV4 {
    payload: [u8; PAYLOAD_SIZE],
}

impl IcmpV4 {
    pub(crate) fn create() -> IcmpV4 {
        let mut payload = [0u8; PAYLOAD_SIZE];
        rand::thread_rng().fill(&mut payload[..]);
        IcmpV4 { payload }
    }

    pub(crate) fn send_one_ping<S>(
        &self,
        socket: &S,
        ipv4: &Ipv4Addr,
        sequence_number: u16,
    ) -> Result<(usize, IpAddr, u16, Instant), PingError>
    where
        S: crate::Socket,
    {
        let ip_addr = IpAddr::V4(*ipv4);
        let addr = std::net::SocketAddr::new(ip_addr, 0);

        let packet = self.new_icmpv4_packet(sequence_number).ok_or(PingError {
            message: "could not create ICMP package".to_owned(),
            source: None,
        })?;

        // TODO(as): use interface and mock for getting time
        let start_time: Instant = Instant::now();
        socket.send_to(packet.packet(), &addr.into())?;

        Ok((PAYLOAD_SIZE, ip_addr, sequence_number, start_time))
    }

    pub(crate) fn try_receive<S>(
        &self,
        socket: &S,
    ) -> std::result::Result<Option<(usize, IpAddr, u16, Instant)>, GenericError>
    where
        S: crate::Socket,
    {
        let mut buf1 = [std::mem::MaybeUninit::<u8>::uninit(); 256];
        match socket.recv_from(&mut buf1) {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
            Ok((n, addr)) => {
                let receive_time: Instant = Instant::now();
                let buf2: Vec<u8> = buf1
                    .iter()
                    .take(n)
                    .map(|&b| unsafe { b.assume_init() })
                    .collect();
                let echo_reply_packet =
                    EchoReplyPacket::new(&buf2).expect("could not initialize echo reply packet");
                let sn = echo_reply_packet.get_sequence_number();
                // To get TTL we will need to create the socket with Protocol::IPV4
                Ok(Some((
                    n,
                    addr.as_socket().expect("logic error").ip(),
                    sn,
                    receive_time,
                )))
            }
        }
    }

    fn new_icmpv4_packet(
        &self,
        sequence_number: u16,
    ) -> Option<MutableEchoRequestPacketV4<'static>> {
        let buf = vec![0u8; EchoRequestPacketV4::minimum_packet_size() + PAYLOAD_SIZE];
        let mut packet = MutableEchoRequestPacketV4::owned(buf)?;
        packet.set_sequence_number(sequence_number);
        packet.set_identifier(0);
        packet.set_icmp_type(IcmpTypes::EchoRequest);
        packet.set_payload(&self.payload);

        packet.set_checksum(0_u16);
        let checksum = pnet_packet::icmp::checksum(&IcmpPacket::new(packet.packet())?);
        packet.set_checksum(checksum);
        Some(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::tests::OnReceive;
    use crate::socket::tests::OnSend;
    use crate::socket::tests::SocketMock;

    #[test]
    fn test_send_one_ping() {
        let socket_mock = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnWouldBlock);
        let icmpv4 = IcmpV4::create();

        let addr = Ipv4Addr::new(127, 0, 0, 1);
        let sequence_number = 1;
        let result = icmpv4.send_one_ping(&socket_mock, &addr, sequence_number);

        assert!(result.is_ok());
        socket_mock
            .should_send_number_of_messages(1)
            .should_send_to_address(&std::net::SocketAddr::new(IpAddr::V4(addr), 0).into());
    }

    #[test]
    fn test_try_receive() {
        let socket_mock = SocketMock::new(OnSend::ReturnDefault, OnReceive::ReturnDefault(1));
        let icmpv4 = IcmpV4::create();

        let result = icmpv4.try_receive(&socket_mock);

        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().is_some());
        let (n, addr, _sn, _receive_time) = result.unwrap().unwrap();
        assert!(n >= EchoReplyPacket::minimum_packet_size());
        assert!(addr == Ipv4Addr::new(127, 0, 0, 1));
        socket_mock.should_receive_number_of_messages(1);
    }
}
