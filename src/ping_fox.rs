use crate::details;
use crate::PingReceive;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

/// The ping-fox configuration structure.
#[allow(clippy::module_name_repetitions)]
pub struct PingFoxConfig {
    /// The type of socket used for network communication.
    pub socket_type: SocketType,
    /// Timeout for `receive` calls on a socket.
    pub timeout: Duration,
    /// Size of the communiation channel used between a [`PingSender`] and a [`PingReceiver`].
    pub channel_size: usize,
}

/// Type of socket used for network communication.
///
/// The socket type also determines whether or not the resulting code needs to be executed with
/// elevated privileges.
#[derive(Clone, Copy)]
pub enum SocketType {
    /// Datagram socket.
    ///
    /// Datagram sockets can be used without elevated privileges.
    DGRAM,
    /// Raw socket.
    ///
    /// Raw sockets need elevated privileges.
    RAW,
}

/// A `PingSentToken` represents an evidence that a ping message has been sent.
// The attribute non_exhaustive prevents construction outside of this crate.
#[non_exhaustive]
pub struct PingSentToken {}

/// Structure used for sending ping echo messages.
pub struct PingSender(details::PingSender<details::icmp::v4::Socket>);
impl PingSender {
    /// Sends a ping echo message and returns a [`PingSentToken`].
    ///
    /// # Arguments
    ///
    /// * `ip` - The address to send the ping to.
    pub fn send_to(&mut self, ip: Ipv4Addr) -> details::PingResult<PingSentToken> {
        self.0.send_to(ip)
    }
}

/// Structure used for receiving ping echo reply messages.
pub struct PingReceiver(details::PingReceiver<details::icmp::v4::Socket>);
impl PingReceiver {
    /// Blocks and waits for ping echo reply message.
    /// Returns the data from the received echo reply message or a structure representing a [`PingReceive::Timeout`].
    ///
    /// # Arguments
    ///
    /// * `token` - A [`PingSentToken`] obtained from a previous call to `PingSender::send_to`.
    pub fn receive(&mut self, token: PingSentToken) -> details::PingResult<PingReceive> {
        self.0.receive(token)
    }
}

/// Principal function in ping-fox. It creates a [`PingSender`] and a [`PingReceiver`].
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
