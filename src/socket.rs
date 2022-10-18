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
