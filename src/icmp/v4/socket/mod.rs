use crate::icmp::v4::Ttl;
use std::{io, time::Duration};

pub(crate) mod dgram_socket;
pub(crate) mod raw_socket;

pub trait Socket: Send + Sync {
    // TODO: can we get rid of the Box inside the Result?
    fn new(timeout: Duration) -> Result<Box<Self>, io::Error>;
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize>;
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)>;
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

    pub(crate) fn default_timeout() -> Duration {
        Duration::from_secs(1)
    }

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

    impl Socket for SocketMock {
        fn new(_timeout: Duration) -> Result<Box<Self>, io::Error> {
            Ok(Box::new(Self::new(
                OnSend::ReturnDefault,
                OnReceive::ReturnDefault(usize::max_value()),
            )))
        }

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
            if buf.len() < EchoReplyPacket::minimum_packet_size() + payload.len() {
                return Err(io::Error::new(io::ErrorKind::Other, "buffer too small"));
            }

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
            for (i, b) in package.packet().iter().enumerate() {
                buf[i] = *b;
            }

            Ok((package.packet_size(), "127.0.0.1".parse::<IpAddr>().unwrap(), Ttl(128)))
        }
    }
}
