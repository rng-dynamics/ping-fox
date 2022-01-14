#![warn(rust_2018_idioms)]

use dns_lookup::{lookup_addr, lookup_host};
use pnet::packet::{
    icmp::{
        echo_reply::EchoReplyPacket,
        echo_request::{
            EchoRequestPacket as EchoRequestPacketV4,
            MutableEchoRequestPacket as MutableEchoRequestPacketV4,
        },
        IcmpPacket, IcmpTypes,
    },
    Packet,
};
use socket2::{Domain, Protocol, Type};
use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr},
    time::{Duration, Instant},
};

#[cfg(test)]
#[macro_use]
extern crate more_asserts;

const PAYLOAD_SIZE: usize = 56;
const PAYLOAD_STR: &str = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr,";

#[derive(Debug, Clone)]
pub struct PingError;

impl From<std::io::Error> for PingError {
    fn from(_: std::io::Error) -> PingError {
        PingError {}
    }
}

#[derive(Default)]
pub struct Ping {
    hostnames: VecDeque<String>,
}

type PingResult<T> = std::result::Result<T, PingError>;

impl Ping {
    pub fn new() -> Ping {
        Ping::default()
    }

    pub fn add_host(&mut self, host: &str) {
        self.hostnames.push_back(host.to_owned());
    }

    fn lookup_host_v4(hostname: &str) -> PingResult<Ipv4Addr> {
        let ips: Vec<std::net::IpAddr> = lookup_host(hostname)?;
        let mut ipv4s: Vec<Ipv4Addr> = ips
            .into_iter()
            .filter_map(|ip| match ip {
                IpAddr::V4(ipv4) => Some(ipv4),
                _ => None,
            })
            .collect();
        if ipv4s.is_empty() {
            Err(PingError {})
        } else {
            Ok(ipv4s.swap_remove(0))
        }
    }

    fn lookup_addr(ip: IpAddr) -> PingResult<String> {
        let hostname = lookup_addr(&ip)?;
        Ok(hostname)
    }

    pub fn run(mut self) -> Vec<PingResult<(String, IpAddr, Duration)>> {
        let mut result = vec![];
        let hostnames = self.hostnames;
        self.hostnames = VecDeque::<String>::new();
        for hostname in hostnames {
            let maybe_ip = Ping::lookup_host_v4(&hostname);
            match maybe_ip {
                Ok(ip) => {
                    let one_result = self.ping_one_ipv4(&ip);
                    result.push(one_result);
                }
                Err(e) => {
                    result.push(Err(e));
                }
            }
        }
        result
    }

    pub fn ping_one_ipv4(&mut self, ipv4: &Ipv4Addr) -> PingResult<(String, IpAddr, Duration)> {
        let sequence_number = 21; // TODO
        let ip_addr = IpAddr::V4(*ipv4);
        let addr = socket2::SockAddr::from(std::net::SocketAddr::new(ip_addr, 0));
        let reverse_lookup = Ping::lookup_addr(ip_addr)?;

        // Using DGRAM to avoid RAW sockets and the need for privileges
        let client = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
        client.connect(&addr).unwrap();
        let packet = Ping::new_icmpv4_packet(sequence_number).ok_or(PingError {})?;

        let start_time = Instant::now();
        client.send(packet.packet())?;

        let mut buf3 = vec![std::mem::MaybeUninit::new(0u8); 256];
        let (n, _addr) = client.recv_from(&mut buf3).unwrap();
        let duration = start_time.elapsed();

        // println!("received {} bytes", n);
        let mut buf4: Vec<u8> = vec![];
        for b in buf3.iter().take(n) {
            buf4.push(unsafe { b.assume_init() });
        }

        let _echo_reply_packet = EchoReplyPacket::owned(buf4).unwrap();

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_addr() {
        let ip_127_0_0_1 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        let hostname = Ping::lookup_addr(ip_127_0_0_1).unwrap();

        assert_eq!(hostname, "localhost");
    }

    #[test]
    fn lookup_host() {
        let ip = Ping::lookup_host_v4("localhost").unwrap();

        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn ping_ipv4_127_0_0_1() {
        let mut ping = Ping::new();
        let addr = Ipv4Addr::new(127, 0, 0, 1);

        let (hostname, ip, dur) = ping.ping_one_ipv4(&addr).unwrap();

        assert_eq!(hostname, "localhost");
        assert_eq!(ip, Ipv4Addr::new(127, 0, 0, 1));
        assert_gt!(dur, Duration::from_secs(0));
    }

    #[test]
    fn ping_localhost() {
        let mut ping = Ping::new();
        ping.add_host("localhost");

        let results = ping.run();
        let (hostname, ip, dur) = results[0].clone().unwrap();

        assert_eq!(hostname, "localhost");
        assert_eq!(ip, Ipv4Addr::new(127, 0, 0, 1));
        assert_gt!(dur, Duration::from_secs(0));
    }

    #[test]
    fn ping_multiple_net() {
        let mut pinger = Ping::new();
        pinger.add_host("example.com");
        pinger.add_host("iana.com");

        let result = pinger.run();

        assert_eq!(result.len(), 2);
        let mut result_iter = result.iter();
        let result_0 = result_iter.next().unwrap().as_ref().unwrap();
        // println!("## result_0 {:?}", result_0);
        assert_eq!(result_0.0, "93.184.216.34");
        assert_eq!(result_0.1, Ipv4Addr::new(93, 184, 216, 34));
        assert_gt!(result_0.2, Duration::from_secs(0));
        let result_1 = result_iter.next().unwrap().as_ref().unwrap();
        // println!("## result_1 {:?}", result_1);
        assert_eq!(result_1.0, "43-8.any.icann.org");
        assert_eq!(result_1.1, Ipv4Addr::new(192, 0, 43, 8));
        assert_gt!(result_1.2, Duration::from_secs(0));
    }
}
