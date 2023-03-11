use crate::event::ping_send_event_channel;
use crate::icmp::v4::Socket;
use crate::GenericError;
use crate::IcmpV4;
use crate::PingDataBuffer;
use crate::PingReceiver;
use crate::PingSender;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

pub type PingResult<T> = std::result::Result<T, GenericError>;

// TODO: write a test that this is not copyable?
// TODO: rename to Token
#[non_exhaustive] // prevent construction outside of this crate
pub struct PingSentEvidence {}

pub enum SocketType {
    DGRAM,
    RAW,
}

// TODO: rename
#[allow(clippy::module_name_repetitions)]
pub struct PingRunnerV2Config<'a> {
    pub ips: &'a [Ipv4Addr],
    pub timeout: Duration,
    pub channel_size: usize,
    // TODO: remove (unused anyway)
    pub socket_type: SocketType,
}

pub fn create<S>(config: &PingRunnerV2Config<'_>) -> PingResult<(PingSender<S>, PingReceiver<S>)>
where
    S: Socket + 'static,
{
    // TODO: can we get rid of the (implicitely used) Box? Leave it for now.
    let socket: S = *S::new(config.timeout)?;
    Ok(create_with_socket(config, socket))
}

fn create_with_socket<S>(
    config: &PingRunnerV2Config<'_>,
    socket: S,
) -> (PingSender<S>, PingReceiver<S>)
where
    S: Socket + 'static,
{
    let ips = config.ips.iter().copied().collect::<VecDeque<Ipv4Addr>>();

    let icmpv4 = Arc::new(IcmpV4::new(socket));
    let (send_event_tx, send_event_rx) = ping_send_event_channel(config.channel_size);
    let ping_data_buffer = PingDataBuffer::new(send_event_rx);

    (
        PingSender::new(icmpv4.clone(), send_event_tx, ips),
        PingReceiver::new(icmpv4, ping_data_buffer),
    )
}

#[cfg(test)]
mod tests {
    use crate::icmp::v4::tests::SocketMock;

    use super::*;

    #[test]
    fn ping_localhost_succeeds() {
        let config = PingRunnerV2Config {
            ips: &[Ipv4Addr::new(127, 0, 0, 1)],
            timeout: Duration::from_secs(1),
            channel_size: 4,
            socket_type: SocketType::DGRAM,
        };

        let (mut ping_sender, mut ping_receiver) = super::create::<SocketMock>(&config).unwrap();
        let mut tokens = ping_sender.send_ping_to_each_address().unwrap();
        let token = tokens.pop().expect("logic error: vec empty");
        let ping_response = ping_receiver.receive_ping(token);
        println!("{ping_response:?}");
        assert!(ping_response.is_ok());
    }
}
