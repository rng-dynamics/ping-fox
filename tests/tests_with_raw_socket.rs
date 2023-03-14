use std::net::Ipv4Addr;
use std::time::Duration;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use ping_fox::{PingFoxConfig, PingReceive, SocketType};

/*
* Note: Raw sockets work only with root privileges.
*/
#[test]
fn test_ping_to_localhost_with_raw_socket() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::ERROR).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let timeout = Duration::from_secs(1);
    let config =
        PingFoxConfig { ips: &[Ipv4Addr::new(127, 0, 0, 1)], timeout, channel_size: 2, socket_type: SocketType::RAW };

    let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config).unwrap();

    let mut tokens = ping_sender.send_ping_to_each_address().unwrap();
    let token = tokens.pop().expect("logic error: vec empty");

    let ping_response = ping_receiver.receive_ping(token);

    assert!(ping_response.is_ok());
    assert!(matches!(ping_response.unwrap(), PingReceive::Data(_)));
}
