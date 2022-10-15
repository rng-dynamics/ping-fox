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
use socket2::{Domain, Protocol, Type};
use std::net::{IpAddr, Ipv4Addr};
use std::result::Result;
use std::time::Instant;

use crate::ping_error::GenericError;
use crate::ping_error::PingError;

const PAYLOAD_SIZE: usize = 56;

pub struct IcmpV4 {
    payload: Vec<u8>,
}

impl IcmpV4 {
    pub(crate) fn create() -> IcmpV4 {
        let mut payload = vec![0u8; PAYLOAD_SIZE];
        rand::thread_rng().fill(&mut payload[..]);
        IcmpV4 { payload }
    }

    pub(crate) fn create_socket() -> Result<socket2::Socket, GenericError> {
        // TODO: make UDP vs raw socket configurable
        Ok(socket2::Socket::new(
            Domain::IPV4,
            Type::DGRAM,
            Some(Protocol::ICMPV4),
        )?)
    }

    pub(crate) fn send_one_ping(
        &self,
        socket: &socket2::Socket,
        ipv4: &Ipv4Addr,
        sequence_number: u16,
    ) -> Result<(usize, IpAddr, u16), PingError> {
        let ip_addr = IpAddr::V4(*ipv4);
        let addr = socket2::SockAddr::from(std::net::SocketAddr::new(ip_addr, 0));
        // let reverse_lookup = utils::lookup_addr(ip_addr)?;

        let packet = self.new_icmpv4_packet(sequence_number).ok_or(PingError {
            message: "could not create ICMP package".to_owned(),
            source: None,
        })?;

        // let start_time = Instant::now();
        socket.send_to(packet.packet(), &addr)?;

        Ok((PAYLOAD_SIZE, ip_addr, sequence_number))
    }

    pub(crate) fn try_receive(
        &self,
        socket: &socket2::Socket,
    ) -> std::result::Result<Option<(usize, IpAddr, u16)>, GenericError> {
        let mut buf3 = vec![std::mem::MaybeUninit::new(0u8); 256];
        let recv_result = socket.recv_from(&mut buf3);
        // let duration = start_time.elapsed();
        if let Err(e) = recv_result {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(None);
            }
            return Err(Box::new(e));
        }
        let (n, addr) = recv_result.unwrap();

        let mut buf4: Vec<u8> = vec![];
        for b in buf3.iter().take(n) {
            buf4.push(unsafe { b.assume_init() });
        }

        let echo_reply_packet =
            EchoReplyPacket::owned(buf4).expect("could not initialize echo reply packet");
        let sn = echo_reply_packet.get_sequence_number();

        Ok(Some((n, addr.as_socket().unwrap().ip(), sn))) // TODO: no unwrap
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
