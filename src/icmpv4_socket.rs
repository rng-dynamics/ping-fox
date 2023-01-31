use std::{io, os::unix::prelude::AsRawFd, time::Duration};

use pnet_packet::{
    icmp::echo_request::MutableEchoRequestPacket as MutableEchoRequestPacketV4, ipv4::Ipv4Packet,
    Packet,
};
use socket2::{Domain, Protocol, Type};

use c_dgram_socket_api::RecvData;

pub(crate) struct Ttl(u8);

impl From<u8> for Ttl {
    fn from(integer: u8) -> Self {
        Ttl(integer)
    }
}

pub(crate) trait IcmpV4Socket: Send + Sync {
    fn send_to(
        &self,
        package: MutableEchoRequestPacketV4<'_>,
        addr: &socket2::SockAddr,
    ) -> io::Result<usize>;

    // TODO: return a EchoReplyPacket ?
    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, std::net::IpAddr, Ttl)>;
}

mod c_dgram_socket_api {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

struct CApiDgramIcmpV4Socket {
    socket: socket2::Socket,
    // socket: std::net::UdpSocket,
}

impl IcmpV4Socket for CApiDgramIcmpV4Socket {
    fn send_to(
        &self,
        package: MutableEchoRequestPacketV4<'_>,
        addr: &socket2::SockAddr,
    ) -> io::Result<usize> {
        self.socket
            .send_to(pnet_packet::Packet::packet(&package), addr)
    }

    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        const BUFFER_LEN: usize = 256;
        let raw_fd: std::ffi::c_int = self.socket.as_raw_fd();
        let mut buffer = [0i8; BUFFER_LEN];
        let buffer_ptr: *mut std::ffi::c_char = buffer.as_mut_ptr();
        let received_data: RecvData =
            unsafe { c_dgram_socket_api::recv_from(raw_fd, buffer_ptr, BUFFER_LEN) };
        let addr_str: String =
            // unsafe {String::from_utf8_unchecked(received_data.addr_str.iter().map(|&c| c as u8).collect())};
                // .expect("cannot convert string");
            str_from_null_terminated_utf8_safe(&received_data.addr_str).into();
        println!("{:?}", addr_str);
        Ok((
            received_data
                .bytes_received
                .try_into()
                .expect("cannot convert integer"),
            addr_str.parse::<std::net::IpAddr>().unwrap().into(),
            received_data
                .ttl
                .try_into()
                .expect("cannot convert integer"),
        ))
    }
}

struct RawSocket {
    socket: socket2::Socket,
}

impl IcmpV4Socket for RawSocket {
    fn send_to(
        &self,
        package: MutableEchoRequestPacketV4<'_>,
        addr: &socket2::SockAddr,
    ) -> io::Result<usize> {
        self.socket.send_to(package.packet(), addr)
    }

    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        // On a RAW socket we get an IP packet.
        let (n, socket_addr) = socket2::Socket::recv_from(&self.socket, buf)?;
        // Unwrape the IP packet
        let buf2: Vec<u8> = buf
            .iter()
            .take(n)
            .map(|&b| unsafe { b.assume_init() })
            .collect();
        let ipv4_packet = Ipv4Packet::new(&buf2).expect("could not initialize IPv4 package");
        let ip_payload = ipv4_packet.payload();
        // Return only the ICMP content
        for (idx, bval) in ip_payload.iter().enumerate() {
            buf[idx] = std::mem::MaybeUninit::new(*bval);
        }
        let ip = *socket_addr.as_socket_ipv4().expect("logic error").ip();
        Ok((ip_payload.len(), std::net::IpAddr::V4(ip), ipv4_packet.get_ttl().into()))
    }
}

pub(crate) fn create_dgram_socket(timeout: Duration) -> Result<impl IcmpV4Socket, io::Error> {
    let socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
    socket
        .set_read_timeout(Some(timeout))
        .expect("could not set socket timeout");
    Ok(CApiDgramIcmpV4Socket { socket })
}

pub(crate) fn create_raw_socket(timeout: Duration) -> Result<impl IcmpV4Socket, io::Error> {
    let socket = socket2::Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    socket
        .set_read_timeout(Some(timeout))
        .expect("could not set socket timeout");
    Ok(RawSocket { socket })
}

fn str_from_null_terminated_utf8_safe(s: &[u8]) -> &str {
    if s.iter().any(|&x| x == 0) {
        unsafe { str_from_null_terminated_utf8(s) }
    } else {
        std::str::from_utf8(s).unwrap()
    }
}

// unsafe: s must contain a null byte
unsafe fn str_from_null_terminated_utf8(s: &[u8]) -> &str {
    std::ffi::CStr::from_ptr(s.as_ptr().cast()).to_str().unwrap()
}
