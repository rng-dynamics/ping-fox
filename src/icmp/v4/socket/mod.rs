pub(crate) use ttl::Ttl;

// TODO: change file/module name of both modules.
pub(crate) mod icmpv4_dgram_socket;
pub(crate) mod icmpv4_raw_socket;

use std::io;

mod ttl;

pub(crate) trait IcmpV4Socket: Send + Sync {
    fn send_to(&self, buf: &[u8], addr: &socket2::SockAddr) -> io::Result<usize>;
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr, Ttl)>;
}
