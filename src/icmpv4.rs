use pnet::packet::icmp::{
    echo_reply::EchoReplyPacket,
    echo_request::{
        EchoRequestPacket as EchoRequestPacketV4,
        MutableEchoRequestPacket as MutableEchoRequestPacketV4,
    },
    IcmpPacket, IcmpTypes,
};
use pnet::packet::Packet;
use socket2::{Domain, Protocol, Type};
use std::{net::{IpAddr, Ipv4Addr}, io::Write};
use std::result::Result;
use std::time::{Duration, Instant};

use crate::ping_error::PingError;
use crate::utils;

const PAYLOAD_SIZE: usize = 56;
// TODO
const PAYLOAD_STR: &str = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr,";

pub(crate) fn ping_one(sequence_number: u16, ipv4: &Ipv4Addr) -> Result<(String, IpAddr, Duration), PingError> {
    let ip_addr = IpAddr::V4(*ipv4);
    let addr = socket2::SockAddr::from(std::net::SocketAddr::new(ip_addr, 0));
    let reverse_lookup = utils::lookup_addr(ip_addr)?;

    // TODO: make UDP vs raw socket configurable
    let client = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
    let packet = new_icmpv4_packet(sequence_number).ok_or(PingError {
        message: "could not create ICMP package".to_owned(),
        source: None,
    })?;

    let start_time = Instant::now();
    client.send_to(packet.packet(), &addr)?;

    let mut buf3 = vec![std::mem::MaybeUninit::new(0u8); 256];
    let (n, _addr) = client.recv_from(&mut buf3).unwrap();
    let duration = start_time.elapsed();

    let mut buf4: Vec<u8> = vec![];
    for b in buf3.iter().take(n) {
        buf4.push(unsafe { b.assume_init() });
    }

    let echo_reply_packet = EchoReplyPacket::owned(buf4).unwrap();

    Ok((reverse_lookup, ip_addr, duration))
}

fn new_icmpv4_packet(sequence_number: u16) -> Option<MutableEchoRequestPacketV4<'static>> {
    let buf = vec![0u8; EchoRequestPacketV4::minimum_packet_size() + PAYLOAD_SIZE];
    let mut packet = MutableEchoRequestPacketV4::owned(buf)?;
    packet.set_sequence_number(sequence_number);
    packet.set_identifier(0);
    packet.set_icmp_type(IcmpTypes::EchoRequest);
    let payload: Vec<u8> = PAYLOAD_STR.bytes().into_iter().take(PAYLOAD_SIZE).collect();
    packet.set_payload(&payload);

    let checksum = pnet::packet::icmp::checksum(&IcmpPacket::new(packet.packet())?);
    packet.set_checksum(checksum);
    Some(packet)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ping_127_0_0_1() {
        let addr = Ipv4Addr::new(127, 0, 0, 1);

        let (hostname, ip, dur) = ping_one(42, &addr).unwrap();

        assert_eq!(hostname, "localhost");
        assert_eq!(ip, Ipv4Addr::new(127, 0, 0, 1));
        assert_gt!(dur, Duration::from_secs(0));
    }
}
