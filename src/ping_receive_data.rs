use crate::icmp::v4::SequenceNumber;
use crate::icmp::v4::Ttl;
use std::net::IpAddr;
use std::time::Instant;

#[derive(PartialEq, Eq)]
pub(crate) struct PingReceiveData {
    pub package_size: usize,
    pub ip_addr: IpAddr,
    pub ttl: Ttl,
    pub sequence_number: SequenceNumber,
    pub receive_time: Instant,
}
