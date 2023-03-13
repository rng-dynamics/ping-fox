use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug)]
pub enum PingReceive {
    Data(PingReceiveData),
    Timeout,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PingReceiveData {
    pub package_size: usize,
    pub ip_addr: IpAddr,
    pub ttl: u8,
    pub sequence_number: u16,
    pub ping_duration: Duration,
}
