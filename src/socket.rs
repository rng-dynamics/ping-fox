use std::io;

pub(crate) trait Socket: Send + Sync {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize>;

    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, socket2::SockAddr)>;
}

impl Socket for socket2::Socket {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
        self.send_to(buf, addr)
    }

    fn recv_from(
        &self,
        buf: &mut [std::mem::MaybeUninit<u8>],
    ) -> io::Result<(usize, socket2::SockAddr)> {
        socket2::Socket::recv_from(self, buf)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::io;
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

    pub(crate) struct SocketMock {
        sent: Mutex<Vec<(Vec<u8>, socket2::SockAddr)>>,
        received_cnt: Mutex<usize>,
    }

    impl crate::Socket for SocketMock {
        fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize> {
            self.sent.lock().unwrap().push((buf.to_vec(), addr.clone()));
            Ok(buf.len())
        }

        fn recv_from(
            &self,
            buf: &mut [std::mem::MaybeUninit<u8>],
        ) -> io::Result<(usize, socket2::SockAddr)> {
            let payload: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF];
            if buf.len() < EchoReplyPacket::minimum_packet_size() + payload.len() {
                return Err(io::Error::new(io::ErrorKind::Other, "buffer too small"));
            }

            let mut received_cnt = self.received_cnt.lock().unwrap();
            *received_cnt += 1;

            let buf2 = vec![0u8; EchoReplyPacket::minimum_packet_size() + payload.len()];
            let mut packet: MutableEchoReplyPacket<'_> =
                MutableEchoReplyPacket::owned(buf2).unwrap();
            packet.set_icmp_type(IcmpType::new(0)); // echo reply
            packet.set_icmp_code(IcmpCode::new(0)); // echo reply
            packet.set_identifier(0xABCD_u16);
            packet.set_sequence_number(1);
            packet.set_payload(&payload);
            packet.set_checksum(0_u16);
            packet.set_checksum(checksum(&IcmpPacket::new(packet.packet()).unwrap()));
            for (i, b) in packet.packet().iter().enumerate() {
                buf[i].write(*b);
            }

            Ok((
                packet.packet_size(),
                "127.0.0.1:12345".parse::<SocketAddr>().unwrap().into(),
            ))
        }
    }

    impl SocketMock {
        pub(crate) fn new() -> Self {
            Self {
                sent: Mutex::new(vec![]),
                received_cnt: Mutex::new(0),
            }
        }

        pub(crate) fn should_send_number_of_messages(&self, n: usize) -> &Self {
            assert!(n == self.sent.lock().unwrap().len());
            self
        }

        pub(crate) fn should_send_to_address(&self, addr: &socket2::SockAddr) -> &Self {
            assert!(self
                .sent
                .lock()
                .unwrap()
                .iter()
                .any(|e| addr.as_socket() == e.1.as_socket()));
            self
        }

        pub(crate) fn should_receive_number_of_messages(&self, n: usize) -> &Self {
            assert!(n == *self.received_cnt.lock().unwrap());
            self
        }
    }
}
