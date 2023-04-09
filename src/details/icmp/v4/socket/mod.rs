use crate::{details::icmp::v4::Ttl, SocketType};
use std::{io, time::Duration};

use super::{DgramSocket, RawSocket};

pub(crate) mod dgram_socket;
pub(crate) mod raw_socket;

pub(crate) trait TSocket: Send + Sync {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize>;
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)>;
}

pub(crate) enum Socket {
    Raw(RawSocket),
    Dgram(DgramSocket),
}

impl Socket {
    pub(crate) fn new(socket_type: SocketType, timeout: Duration) -> Result<Self, io::Error> {
        match socket_type {
            SocketType::DGRAM => Ok(Socket::Dgram(DgramSocket::new(timeout)?)),
            SocketType::RAW => Ok(Socket::Raw(RawSocket::new(timeout)?)),
        }
    }
}

impl TSocket for Socket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        match self {
            Socket::Dgram(socket) => socket.send_to(buf, addr),
            Socket::Raw(socket) => socket.send_to(buf, addr),
        }
    }

    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)> {
        match self {
            Socket::Dgram(socket) => socket.recv_from(buf),
            Socket::Raw(socket) => socket.recv_from(buf),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use std::net::IpAddr;
    use std::sync::Arc;
    use std::sync::Mutex;

    use pnet_packet::icmp::checksum;
    use pnet_packet::icmp::echo_reply::EchoReplyPacket;
    use pnet_packet::icmp::echo_reply::MutableEchoReplyPacket;
    use pnet_packet::icmp::IcmpCode;
    use pnet_packet::icmp::IcmpPacket;
    use pnet_packet::icmp::IcmpType;
    use pnet_packet::Packet;
    use pnet_packet::PacketSize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) enum OnSend {
        ReturnErr,
        ReturnDefault,
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub(crate) enum OnReceive {
        ReturnWouldBlock,
        ReturnDefault(usize),
    }

    type VecOfBuffersAndAddresses = Arc<Mutex<Vec<(Vec<u8>, IpAddr)>>>;

    pub(crate) struct SocketMock {
        on_send: OnSend,
        on_receive: Arc<Mutex<OnReceive>>,
        sent: VecOfBuffersAndAddresses,
        received_cnt: Arc<Mutex<u16>>,
    }

    impl Clone for SocketMock {
        fn clone(&self) -> Self {
            SocketMock {
                on_send: self.on_send,
                on_receive: self.on_receive.clone(),
                sent: self.sent.clone(),
                received_cnt: self.received_cnt.clone(),
            }
        }
    }

    impl SocketMock {
        pub(crate) fn new(on_send: OnSend, on_receive: OnReceive) -> Self {
            Self {
                on_send,
                on_receive: Arc::new(Mutex::new(on_receive)),
                sent: Arc::new(Mutex::new(vec![])),
                received_cnt: Arc::new(Mutex::new(0)),
            }
        }

        pub(crate) fn new_default() -> Self {
            Self::new(OnSend::ReturnDefault, OnReceive::ReturnDefault(usize::max_value()))
        }

        pub(crate) fn should_send_number_of_messages(&self, n: usize) -> &Self {
            assert!(n == self.sent.lock().unwrap().len());
            self
        }

        pub(crate) fn should_send_to_address(&self, addr: &IpAddr) -> &Self {
            assert!(self.sent.lock().unwrap().iter().any(|e| *addr == e.1));
            self
        }

        pub(crate) fn should_receive_number_of_messages(&self, n: u16) -> &Self {
            assert!(n == *self.received_cnt.lock().unwrap());
            self
        }
    }

    impl TSocket for SocketMock {
        fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
            if self.on_send == OnSend::ReturnErr {
                return Err(io::Error::new(io::ErrorKind::Other, "simulating error in mock"));
            }
            self.sent.lock().unwrap().push((
                buf.to_vec(),
                addr.as_socket()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "error in extracting IP address from SockAddr"))?
                    .ip(),
            ));
            Ok(buf.len())
        }

        fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, IpAddr, Ttl)> {
            let on_receive: OnReceive = *self.on_receive.lock().unwrap();
            match on_receive {
                OnReceive::ReturnWouldBlock => {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock, "simulating would-block in mock"));
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

            let mut received_cnt = self.received_cnt.lock().unwrap();
            *received_cnt += 1;

            let buf2 = vec![0u8; EchoReplyPacket::minimum_packet_size() + payload.len()];
            let mut package: MutableEchoReplyPacket<'_> = MutableEchoReplyPacket::owned(buf2).unwrap();
            package.set_icmp_type(IcmpType::new(0)); // echo reply
            package.set_icmp_code(IcmpCode::new(0)); // echo reply
            package.set_identifier(0xABCD_u16);
            package.set_sequence_number(*received_cnt);
            package.set_payload(&payload);
            package.set_checksum(0_u16);
            package.set_checksum(checksum(&IcmpPacket::new(package.packet()).unwrap()));
            let package_bytes: &[u8] = package.packet();
            if buf.len() < package_bytes.len() {
                return Err(io::Error::new(io::ErrorKind::Other, "buffer too small"));
            }
            buf[..package_bytes.len()].copy_from_slice(package_bytes);

            Ok((package.packet_size(), "127.0.0.1".parse::<IpAddr>().unwrap(), Ttl(128)))
        }
    }
}
