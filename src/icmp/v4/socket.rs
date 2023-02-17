use pnet_packet::{ipv4::Ipv4Packet, Packet};
use socket2::{Domain, Protocol, Type};
use std::{io, os::unix::prelude::AsRawFd, time::Duration};

use c_dgram_socket_api::IcmpData;

pub(crate) struct Ttl(u8);

impl From<u8> for Ttl {
    fn from(integer: u8) -> Self {
        Ttl(integer)
    }
}

pub(crate) trait IcmpV4Socket: Send + Sync {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize>;

    // TODO: return a EchoReplyPacket ? (or change`send_to` to use byte array)
    fn recv_from(
        &self,
        // buf: &mut [std::mem::MaybeUninit<u8>],
        buf: &mut [u8],
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
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.socket
            // .send_to(pnet_packet::Packet::packet(&package), addr)
            .send_to(buf, addr)
    }

    fn recv_from(
        &self,
        // buf: &mut [std::mem::MaybeUninit<u8>],
        buf: &mut [u8],
    ) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        // const BUFFER_LEN: usize = 64; // TODO: remove magic
        // let mut data_buffer = [0u8; BUFFER_LEN];
        // let mut icmp_data: IcmpData = IcmpData {
        //     data_buffer: data_buffer.as_mut_ptr(),
        //     data_buffer_size: data_buffer.len() as u64,
        //     n_data_bytes_received: 0,
        //     ttl: 0,
        //     addr_str: [0u8; 46],
        // };
        let mut icmp_data: IcmpData = IcmpData {
            data_buffer: buf.as_mut_ptr(),
            data_buffer_size: buf.len() as u64,
            n_data_bytes_received: 0,
            ttl: 0,
            addr_str: [0u8; 46],
        };

        let raw_fd: std::ffi::c_int = self.socket.as_raw_fd();
        let n_bytes_received =
            unsafe { c_dgram_socket_api::recv_from(raw_fd, std::ptr::addr_of_mut!(icmp_data)) };
        if n_bytes_received < 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("error {} reading from socket", n_bytes_received),
            ));
        }
        if n_bytes_received == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "socket closed"));
        }
        let addr_str: String = str_from_null_terminated_utf8_safe(&icmp_data.addr_str).to_string();
        println!("{:?}", addr_str);
        Ok((
            icmp_data.n_data_bytes_received,
            addr_str
                .parse::<std::net::IpAddr>()
                .expect("error reading IP address"),
            icmp_data.ttl.try_into().expect("error decoding TTL"),
        ))
    }
}

struct RawSocket {
    socket: socket2::Socket,
}

impl IcmpV4Socket for RawSocket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(
        &self,
        // buf: &mut [std::mem::MaybeUninit<u8>],
        buf: &mut [u8],
    ) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        let mut recv_buf = [0u8; 256];

        // Socket2 gives a safety guaranty which allows us to do an unsafe cast from `&mut [u8]`
        // to `&mut [std::mem::MaybeUninit<u8>]`. In fact, even if we use MaybeUninit here we have
        // to use unsafe somewhere to copy the data out of MaybeUninit.
        // https://docs.rs/socket2/0.4.7/socket2/struct.Socket.html#method.recv
        //
        // On a RAW socket we get an IP packet.
        let (_, socket_addr) = socket2::Socket::recv_from(&self.socket, unsafe {
            // &mut *(&mut recv_buf as *mut [u8] as *mut [std::mem::MaybeUninit<u8>])
            &mut *(std::ptr::addr_of_mut!(recv_buf) as *mut [u8]
                as *mut [std::mem::MaybeUninit<u8>])
        })?;
        let ipv4_packet = Ipv4Packet::new(&recv_buf).expect("could not initialize IPv4 package");
        let ip_payload = &ipv4_packet.payload();
        // Return only the ICMP content
        for (idx, bval) in ip_payload.iter().enumerate() {
            buf[idx] = *bval;
        }
        let ip = *socket_addr.as_socket_ipv4().expect("logic error").ip();
        Ok((
            ip_payload.len(),
            std::net::IpAddr::V4(ip),
            ipv4_packet.get_ttl().into(),
        ))
    }
}

pub(crate) fn create_dgram_socket(timeout: Duration) -> Result<impl IcmpV4Socket, io::Error> {
    tracing::trace!("creating icmpv4_socket::CApiDgramIcmpV4Socket");
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
    if s.iter().any(|&x| x == 0u8) {
        unsafe { str_from_null_terminated_utf8(s) }
    } else {
        std::str::from_utf8(s).unwrap()
    }
}

// unsafe: s must contain a null byte
unsafe fn str_from_null_terminated_utf8(s: &[u8]) -> &str {
    std::ffi::CStr::from_ptr(s.as_ptr().cast())
        .to_str()
        .unwrap()
}
