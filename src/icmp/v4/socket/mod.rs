use std::io;
use ttl::Ttl;

pub(crate) mod dgram_socket;
pub(crate) mod raw_socket;
mod ttl;

pub(crate) trait Socket: Send + Sync {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize>;
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)>;
}
