use crate::icmp::v4::Socket;
use crate::records::ping_send_record_channel;
use crate::IcmpV4;
use crate::PingDataBuffer;
use crate::PingReceiver;
use crate::PingResult;
use crate::PingSender;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

// The attribute non_exhaustive prevents construction outside of this crate.
#[non_exhaustive]
pub struct PingSentToken {}

#[allow(clippy::module_name_repetitions)]
pub struct PingFoxConfig<'a> {
    pub ips: &'a [Ipv4Addr],
    pub timeout: Duration,
    pub channel_size: usize,
}

pub fn create<S>(config: &PingFoxConfig<'_>) -> PingResult<(PingSender<S>, PingReceiver<S>)>
where
    S: Socket + 'static,
{
    let socket: S = *S::new(config.timeout)?;
    Ok(create_with_socket(config, socket))
}

fn create_with_socket<S>(config: &PingFoxConfig<'_>, socket: S) -> (PingSender<S>, PingReceiver<S>)
where
    S: Socket + 'static,
{
    let ips = config.ips.iter().copied().collect::<VecDeque<Ipv4Addr>>();

    let icmpv4 = Arc::new(IcmpV4::new(socket));
    let (send_record_tx, send_record_rx) = ping_send_record_channel(config.channel_size);
    let ping_data_buffer = PingDataBuffer::new(send_record_rx);

    (
        PingSender::new(icmpv4.clone(), send_record_tx, ips),
        PingReceiver::new(icmpv4, ping_data_buffer),
    )
}

#[cfg(test)]
mod tests {
    use crate::icmp::v4::tests::SocketMock;

    use super::*;

    #[test]
    fn ping_localhost_succeeds() {
        let config = PingFoxConfig { ips: &[Ipv4Addr::new(127, 0, 0, 1)], timeout: Duration::from_secs(1), channel_size: 4 };

        let (mut ping_sender, mut ping_receiver) = super::create::<SocketMock>(&config).unwrap();
        let mut tokens = ping_sender.send_ping_to_each_address().unwrap();
        let token = tokens.pop().expect("logic error: vec empty");
        let ping_response = ping_receiver.receive_ping(token);
        println!("{ping_response:?}");
        assert!(ping_response.is_ok());
    }
}
