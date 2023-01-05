use std::io;
use std::time::Duration;

use pnet_packet::{Packet, ipv4::Ipv4Packet};

use socket2::{Domain, Protocol, Type};

pub(crate) trait Socket: Send + Sync {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize>;

    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, socket2::SockAddr)>;
}

struct DgramSocket {
    socket: socket2::Socket,
}

struct RawSocket {
    socket: socket2::Socket,
}

impl Socket for DgramSocket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, socket2::SockAddr)> {
            socket2::Socket::recv_from(&self.socket, buf)
    }
}

impl Socket for RawSocket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, socket2::SockAddr)> {
        // On a RAW socket we get an IP packet.
        let (n, addr) = socket2::Socket::recv_from(&self.socket, buf)?;
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
        Ok((ip_payload.len(), addr))
    }
}

pub(crate) fn create_socket2_dgram_socket(timeout: Duration) -> Result<impl Socket, io::Error> {
    let socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
    socket
        .set_read_timeout(Some(timeout))
        .expect("could not set socket timeout");
    Ok(DgramSocket{ socket })
}

pub(crate) fn create_socket2_raw_socket(timeout: Duration) -> Result<impl Socket, io::Error> {
    let socket = socket2::Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    socket
        .set_read_timeout(Some(timeout))
        .expect("could not set socket timeout");
    Ok(RawSocket{ socket })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use std::net::IpAddr;
    use std::net::SocketAddr;
    use std::sync::Mutex;

    use pnet_packet::icmp::checksum;
    use pnet_packet::icmp::echo_reply::EchoReplyPacket;
    use pnet_packet::icmp::echo_reply::MutableEchoReplyPacket;
    use pnet_packet::icmp::IcmpCode;
    use pnet_packet::icmp::IcmpPacket;
    use pnet_packet::icmp::IcmpType;
    use pnet_packet::Packet;
    use pnet_packet::PacketSize;

    #[derive(PartialEq, Eq)]
    pub(crate) enum OnSend {
        ReturnErr,
        ReturnDefault,
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub(crate) enum OnReceive {
        ReturnWouldBlock,
        ReturnDefault(usize),
    }

    pub(crate) struct SocketMock {
        on_send: OnSend,
        on_receive: Mutex<OnReceive>,
        sent: Mutex<Vec<(Vec<u8>, IpAddr)>>,
        received_cnt: Mutex<usize>,
    }

    impl SocketMock {
        pub(crate) fn new(on_send: OnSend, on_receive: OnReceive) -> Self {
            Self {
                on_send,
                on_receive: Mutex::new(on_receive),
                sent: Mutex::new(vec![]),
                received_cnt: Mutex::new(0),
            }
        }

        pub(crate) fn should_send_number_of_messages(&self, n: usize) -> &Self {
            assert!(n == self.sent.lock().unwrap().len());
            self
        }

        pub(crate) fn should_send_to_address(&self, addr: &IpAddr) -> &Self {
            assert!(self.sent.lock().unwrap().iter().any(|e| *addr == e.1));
            self
        }

        pub(crate) fn should_receive_number_of_messages(&self, n: usize) -> &Self {
            assert!(n == *self.received_cnt.lock().unwrap());
            self
        }
    }

    impl crate::Socket for SocketMock {
        fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
            if self.on_send == OnSend::ReturnErr {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "simulating error in mock",
                ));
            }
            self.sent.lock().unwrap().push((
                buf.to_vec(),
                addr.as_socket()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            "error in extracting IP address from SockAddr",
                        )
                    })?
                    .ip(),
            ));
            Ok(buf.len())
        }

        fn recv_from(
            &self,
            buf: &mut [std::mem::MaybeUninit<u8>],
        ) -> io::Result<(usize, socket2::SockAddr)> {
            let on_receive: OnReceive = *self.on_receive.lock().unwrap();
            match on_receive {
                OnReceive::ReturnWouldBlock => {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "simulating would-block in mock",
                    ));
                }
                OnReceive::ReturnDefault(cnt) => {
                    *self.on_receive.lock().unwrap() = if cnt <= 1 {
                        OnReceive::ReturnWouldBlock
                    } else {
                        OnReceive::ReturnDefault(cnt - 1)
                    };
                }
            };

            let payload: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF];
            if buf.len() < EchoReplyPacket::minimum_packet_size() + payload.len() {
                return Err(io::Error::new(io::ErrorKind::Other, "buffer too small"));
            }

            let mut received_cnt = self.received_cnt.lock().unwrap();
            *received_cnt += 1;

            let buf2 = vec![0u8; EchoReplyPacket::minimum_packet_size() + payload.len()];
            let mut package: MutableEchoReplyPacket<'_> =
                MutableEchoReplyPacket::owned(buf2).unwrap();
            package.set_icmp_type(IcmpType::new(0)); // echo reply
            package.set_icmp_code(IcmpCode::new(0)); // echo reply
            package.set_identifier(0xABCD_u16);
            package.set_sequence_number(0);
            package.set_payload(&payload);
            package.set_checksum(0_u16);
            package.set_checksum(checksum(&IcmpPacket::new(package.packet()).unwrap()));
            for (i, b) in package.packet().iter().enumerate() {
                buf[i].write(*b);
            }

            Ok((
                package.packet_size(),
                "127.0.0.1:12345".parse::<SocketAddr>().unwrap().into(),
            ))
        }
    }
}
