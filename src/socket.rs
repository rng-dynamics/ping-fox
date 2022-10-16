use std::io;
use std::net::SocketAddr;
use std::time::Duration;

pub(crate) trait Socket: Send + Sync {
    fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize>;
    fn try_recv_from(
        &self,
        buf: &mut [u8],
        timeout: &Duration,
    ) -> io::Result<Option<(usize, SocketAddr)>>;
}

impl Socket for socket2::Socket {
    fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        let sock_addr: socket2::SockAddr = socket2::SockAddr::from(*addr);
        self.send_to(buf, &sock_addr)
    }

    fn try_recv_from(
        &self,
        buf: &mut [u8],
        timeout: &Duration,
    ) -> io::Result<Option<(usize, SocketAddr)>> {
        match self.read_timeout() {
            Ok(None) => {
                self.set_read_timeout(Some(*timeout))
                    .expect("could not set socket timeout");
            }
            Ok(Some(socket_timeout)) => {
                if socket_timeout != *timeout {
                    self.set_read_timeout(Some(*timeout))
                        .expect("could not set socket timeout");
                }
            }
            Err(e) => {
                println!("log ERROR: could not read timeout of socket");
                return Err(e);
            }
        }

        let mut buf1 = vec![std::mem::MaybeUninit::new(0u8); buf.len()];
        match self.recv_from(&mut buf1) {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
            Ok((n, addr)) => {
                for (i, b) in buf.iter_mut().enumerate().take(n) {
                    *b = unsafe { buf1.get(i).expect("logic error").assume_init() };
                }
                Ok(Some((n, addr.as_socket().expect("logic error"))))
            }
        }
    }
}
