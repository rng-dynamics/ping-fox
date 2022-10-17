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
use std::io;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::{IpAddr, Ipv4Addr};
use std::result::Result;
use std::time::Duration;

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
    ) -> Result<(usize, IpAddr, u16), PingError>
    where
        S: crate::Socket,
    {
        let ip_addr = IpAddr::V4(*ipv4);
        let addr = std::net::SocketAddr::new(ip_addr, 0);

        let packet = self.new_icmpv4_packet(sequence_number).ok_or(PingError {
            message: "could not create ICMP package".to_owned(),
            source: None,
        })?;

        // let start_time = Instant::now();
        socket.send_to(packet.packet(), &addr)?;

        Ok((PAYLOAD_SIZE, ip_addr, sequence_number))
    }

    pub(crate) fn try_receive<S>(
        &self,
        socket: &S,
    ) -> std::result::Result<Option<(usize, IpAddr, u16)>, GenericError>
    where
        S: crate::Socket,
    {
        let mut buf = [0u8; 256];
        match socket.try_recv_from(&mut buf, &Duration::from_millis(100)) {
            Ok(None) => Ok(None),
            Err(e) => Err(Box::new(e)),
            Ok(Some((n, addr))) => {
                // let duration = start_time.elapsed();
                let echo_reply_packet = EchoReplyPacket::new(&buf[..n])
                    .expect("could not initialize echo reply packet");
                let sn = echo_reply_packet.get_sequence_number();

                Ok(Some((n, addr.ip(), sn)))
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

        let checksum = pnet_packet::icmp::checksum(&IcmpPacket::new(packet.packet())?);
        packet.set_checksum(checksum);
        Some(packet)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Mutex;

    struct SocketMock {
        sent: Mutex<Vec<(Vec<u8>, SocketAddr)>>,
        received: Mutex<Vec<(Vec<u8>, Duration)>>,
    }

    impl SocketMock {
        fn new() -> Self {
            Self {
                sent: Mutex::new(vec![]),
                received: Mutex::new(vec![]),
            }
        }

        fn should_send_number_of_messages(&self, n: usize) -> &Self {
            assert!(n == self.sent.lock().unwrap().len());
            self
        }

        fn should_send_to_address(&self, addr: &SocketAddr) -> &Self {
            assert!(self.sent.lock().unwrap().iter().any(|e| *addr == e.1));
            self
        }
    }

    impl crate::Socket for SocketMock {
        fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
            self.sent.lock().unwrap().push((buf.to_vec(), *addr));
            Ok(buf.len())
        }

        fn try_recv_from(
            &self,
            buf: &mut [u8],
            timeout: &Duration,
        ) -> io::Result<Option<(usize, SocketAddr)>> {
            self.received.lock().unwrap().push((buf.to_vec(), *timeout));
            Ok(Some((
                64,
                std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 12345)),
            )))
        }
    }

    #[test]
    fn test_send_one_ping() {
        let socket_mock = SocketMock::new();

        let icmpv4 = IcmpV4::create();

        let addr = Ipv4Addr::new(127, 0, 0, 1);
        let sequence_number = 1;
        let result = icmpv4.send_one_ping(&socket_mock, &addr, sequence_number);

        assert!(result.is_ok());
        socket_mock
            .should_send_number_of_messages(1)
            .should_send_to_address(&std::net::SocketAddr::new(IpAddr::V4(addr), 0));
    }
}
