use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug)]
pub enum PingReceiveResult {
    Data(PingReceiveResultData),
    Timeout,
}

// TODO: rename (but PingReceiveData is already taken by another struct).
// TODO: rename file (to fit)
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PingReceiveResultData {
    pub package_size: usize,
    pub ip_addr: IpAddr,
    pub ttl: u8,
    pub sequence_number: u16,
    pub ping_duration: Duration,
}
