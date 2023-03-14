use crate::icmp::v4::TSocket;
use crate::records::ping_send_record_channel;
use crate::IcmpV4;
use crate::PingDataBuffer;
use crate::PingReceive;
use crate::PingResult;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

#[allow(clippy::module_name_repetitions)]
pub struct PingFoxConfig<'a> {
    pub ips: &'a [Ipv4Addr],
    pub timeout: Duration,
    pub channel_size: usize,
    pub socket_type: SocketType,
}

#[derive(Clone, Copy)]
pub enum SocketType {
    DGRAM,
    RAW,
}

// The attribute non_exhaustive prevents construction outside of this crate.
#[non_exhaustive]
pub struct PingSentToken {}

pub struct PingSender(crate::PingSender<crate::icmp::v4::Socket>);
impl PingSender {
    pub fn send_ping_to_each_address(&mut self) -> PingResult<Vec<PingSentToken>> {
        self.0.send_ping_to_each_address()
    }
}
pub struct PingReceiver(crate::PingReceiver<crate::icmp::v4::Socket>);
impl PingReceiver {
    pub fn receive_ping(&mut self, token: PingSentToken) -> PingResult<PingReceive> {
        self.0.receive_ping(token)
    }
}

pub fn create(config: &PingFoxConfig<'_>) -> PingResult<(PingSender, PingReceiver)> {
    let socket: crate::icmp::v4::Socket = crate::icmp::v4::Socket::new(config.socket_type, config.timeout)?;
    let (sender, receiver) = create_with_socket::<crate::icmp::v4::Socket>(socket, config.ips, config.channel_size);
    Ok((PingSender(sender), PingReceiver(receiver)))
}

fn create_with_socket<S>(socket: S, ips: &[Ipv4Addr], channel_size: usize) -> (crate::PingSender<S>, crate::PingReceiver<S>)
where
    S: TSocket + 'static,
{
    let ips = ips.iter().copied().collect::<VecDeque<Ipv4Addr>>();
    let icmpv4 = Arc::new(IcmpV4::new(socket));
    let (send_record_tx, send_record_rx) = ping_send_record_channel(channel_size);
    let ping_data_buffer = PingDataBuffer::new(send_record_rx);
    (
        crate::PingSender::new(icmpv4.clone(), send_record_tx, ips),
        crate::PingReceiver::new(icmpv4, ping_data_buffer),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icmp::v4::tests::SocketMock;

    #[test]
    fn ping_localhost_succeeds() {
        let ips = &[Ipv4Addr::new(127, 0, 0, 1)];
        let channel_size = 4;
        let socket = SocketMock::new_default();

        let (mut ping_sender, mut ping_receiver) = super::create_with_socket(socket, ips, channel_size);
        let mut tokens = ping_sender.send_ping_to_each_address().unwrap();
        let token = tokens.pop().expect("logic error: vec empty");
        let ping_response = ping_receiver.receive_ping(token);

        assert!(ping_response.is_ok());
    }
}
