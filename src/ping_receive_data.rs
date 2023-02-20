use crate::Ttl;
use std::net::IpAddr;
use std::time::Instant;

#[derive(PartialEq, Eq)]
pub(crate) struct PingReceiveData {
    pub package_size: usize,
    pub ip_addr: IpAddr,
    pub ttl: Ttl,
    pub sequence_number: u16,
    pub receive_time: Instant,
}
