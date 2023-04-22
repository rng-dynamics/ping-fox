use crate::details;
use crate::PingReceive;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

#[allow(clippy::module_name_repetitions)]
pub struct PingFoxConfig {
    pub socket_type: SocketType,
    pub timeout: Duration,
    pub channel_size: usize,
}

#[derive(Clone, Copy)]
pub enum SocketType {
    DGRAM,
    RAW,
}

// The attribute non_exhaustive prevents construction outside of this crate.
#[non_exhaustive]
pub struct PingSentToken {}

pub struct PingSender(details::PingSender<details::icmp::v4::Socket>);
impl PingSender {
    pub fn send_to(&mut self, ip: Ipv4Addr) -> details::PingResult<PingSentToken> {
        self.0.send_to(ip)
    }
}
pub struct PingReceiver(details::PingReceiver<details::icmp::v4::Socket>);
impl PingReceiver {
    pub fn receive(&mut self, token: PingSentToken) -> details::PingResult<PingReceive> {
        self.0.receive(token)
    }
}

pub fn create(config: &PingFoxConfig) -> details::PingResult<(PingSender, PingReceiver)> {
    let socket = details::icmp::v4::Socket::new(config.socket_type, config.timeout)?;
    let (sender, receiver) = create_with_socket::<details::icmp::v4::Socket>(socket, config.channel_size);
    Ok((PingSender(sender), PingReceiver(receiver)))
}

fn create_with_socket<S>(socket: S, channel_size: usize) -> (details::PingSender<S>, details::PingReceiver<S>)
where
    S: details::icmp::v4::TSocket + 'static,
{
    let icmpv4 = Arc::new(details::icmp::v4::IcmpV4::new(socket));
    let (send_record_tx, send_record_rx) = details::records::ping_send_record_channel(channel_size);
    let ping_data_buffer = details::PingDataBuffer::new(send_record_rx);
    (
        details::PingSender::new(icmpv4.clone(), send_record_tx),
        details::PingReceiver::new(icmpv4, ping_data_buffer),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use details::icmp::v4::tests::SocketMock;

    #[test]
    fn ping_localhost_succeeds() {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let channel_size = 4;
        let socket = SocketMock::new_default();

        let (mut ping_sender, mut ping_receiver) = super::create_with_socket(socket, channel_size);
        let token = ping_sender.send_to(ip).unwrap();
        let ping_response = ping_receiver.receive(token);

        assert!(ping_response.is_ok());

        let token = ping_sender.send_to(ip).unwrap();
        let ping_response = ping_receiver.receive(token);

        assert!(ping_response.is_ok());
    }
}
