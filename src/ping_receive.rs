use std::net::IpAddr;
use std::time::Duration;

/// Structure representing ping receive cases.
#[derive(Debug)]
pub enum PingReceive {
    /// Case represeting the data from a received echo reply message.
    Data(PingReceiveData),
    /// Case representing a timeout on an attempt to receive an echo reply message.
    Timeout,
}

/// Structure represeting a received echo reply message.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PingReceiveData {
    /// The size of the payload in the received reply message.
    pub package_size: usize,
    /// The IP address of the host which sent the reply.
    pub ip_addr: IpAddr,
    /// The time to live (TTL) of the received reply message.
    pub ttl: u8,
    /// The sequence number of the echo reply.
    pub sequence_number: u16,
    /// The measured duration between sending the echo message and receiving the reply.
    pub ping_duration: Duration,
}
