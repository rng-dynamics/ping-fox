use super::{Socket, Ttl};
use socket2::{Domain, Protocol, Type};
use std::{io, os::unix::prelude::AsRawFd, time::Duration};

mod c_icmp_dgram_api {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub(crate) struct CDgramSocket {
    socket: socket2::Socket,
}

impl CDgramSocket {
    pub(crate) fn create(timeout: Duration) -> Result<impl Socket, io::Error> {
        tracing::trace!("creating icmpv4_socket::CApiDgramIcmpV4Socket");
        let socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
        socket
            .set_read_timeout(Some(timeout))
            .expect("could not set socket timeout");
        Ok(CDgramSocket { socket })
    }
}

impl Socket for CDgramSocket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        let mut icmp_data = c_icmp_dgram_api::IcmpData {
            data_buffer: buf.as_mut_ptr(),
            data_buffer_size: buf.len() as u64,
            n_data_bytes_received: 0,
            ttl: 0,
            addr_str: [0u8; 46],
        };

        let raw_fd: std::ffi::c_int = self.socket.as_raw_fd();
        let n_bytes_received =
            unsafe { c_icmp_dgram_api::recv_from(raw_fd, std::ptr::addr_of_mut!(icmp_data)) };
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    const BUFFER_LEN: usize = 256;

    #[test]
    fn recv_from_succeeds() {
        let icmpv4 = crate::IcmpV4::create();
        let package = icmpv4.new_icmpv4_package(0).unwrap();

        let dgram_socket =
            CDgramSocket::create(super::super::default_timeout()).expect("error creating socket");

        dgram_socket
            .send_to(
                pnet_packet::Packet::packet(&package),
                // &"127.0.0.1:7".parse::<SocketAddr>().unwrap().into(),
                // &"8.8.8.8:7".parse::<SocketAddr>().unwrap().into(),
                &"127.0.0.1:0".parse::<SocketAddr>().unwrap().into(),
            )
            .unwrap();

        let mut buffer = [0u8; BUFFER_LEN];

        let result = dgram_socket.recv_from(&mut buffer);
        assert!(result.is_ok());
        let (n_bytes, _addr, _ttl) = result.unwrap();
        assert!(n_bytes > 0);
    }
}
