use super::Socket;
use crate::Ttl;
use pnet_packet::{ipv4::Ipv4Packet, Packet};
use socket2::{Domain, Protocol, Type};
use std::{io, time::Duration};

pub(crate) struct RawSocket {
    socket: socket2::Socket,
}

impl RawSocket {
    pub(crate) fn create(timeout: Duration) -> Result<impl Socket, io::Error> {
        let socket = socket2::Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
        socket
            .set_read_timeout(Some(timeout))
            .expect("could not set socket timeout");
        Ok(RawSocket { socket })
    }
}

impl Socket for RawSocket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        let mut recv_buf = [0u8; 256];

        // Socket2 gives a safety guaranty which allows us to do an unsafe cast from `&mut [u8]`
        // to `&mut [std::mem::MaybeUninit<u8>]`. In fact, even if we use MaybeUninit here we have
        // to use unsafe somewhere to copy the data out of MaybeUninit.
        // https://docs.rs/socket2/0.4.7/socket2/struct.Socket.html#method.recv
        //
        // On a RAW socket we get an IP packet.
        let (_, socket_addr) = socket2::Socket::recv_from(&self.socket, unsafe {
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
