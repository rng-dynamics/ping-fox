use super::TSocket;
use crate::details::icmp::v4::Ttl;
use socket2::{Domain, Protocol, Type};
use std::{io, os::unix::prelude::AsRawFd, time::Duration};

mod c_icmp_dgram {
    #![allow(clippy::pedantic)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub(crate) struct DgramSocket {
    socket: socket2::Socket,
}

impl DgramSocket {
    pub(crate) fn new(timeout: Duration) -> Result<Self, io::Error> {
        tracing::trace!("creating DgramSocket");
        let socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
        socket.set_read_timeout(Some(timeout)).expect("could not set socket timeout");
        Ok(DgramSocket { socket })
    }
}

impl TSocket for DgramSocket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        let mut icmp_data = c_icmp_dgram::IcmpData {
            data_buffer: buf.as_mut_ptr(),
            data_buffer_size: buf.len() as u64,
            n_data_bytes_received: 0,
            ttl: 0,
            addr_str: [0u8; 46],
        };

        let raw_fd: std::ffi::c_int = self.socket.as_raw_fd();
        let n_bytes_received = unsafe { c_icmp_dgram::recv_from(raw_fd, std::ptr::addr_of_mut!(icmp_data)) };
        if n_bytes_received < 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("error {n_bytes_received} reading from socket"),
            ));
        }
        if n_bytes_received == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "socket closed"));
        }
        let addr_str: String = str_from_null_terminated_utf8_safe(&icmp_data.addr_str).to_string();
        Ok((
            icmp_data.n_data_bytes_received,
            addr_str.parse::<std::net::IpAddr>().expect("error reading IP address"),
            icmp_data.ttl.try_into().expect("error decoding TTL"),
        ))
    }
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
    std::ffi::CStr::from_ptr(s.as_ptr().cast()).to_str().unwrap()
}
