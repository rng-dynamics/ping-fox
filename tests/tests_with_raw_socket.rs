use ping_fox::{PingFoxConfig, PingReceive, SocketType};
use std::time::Duration;
use std::{net::Ipv4Addr, sync::Once};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static SETUP: Once = Once::new();

fn setup() {
    SETUP.call_once(|| {
        let subscriber = FmtSubscriber::builder().with_max_level(Level::ERROR).finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    });
}

/*
* Note: Raw sockets work only with root privileges.
*/
#[test]
fn test_ping_to_localhost_with_raw_socket() {
    setup();

    let timeout = Duration::from_secs(1);
    let config = PingFoxConfig { timeout, channel_size: 2, socket_type: SocketType::RAW };

    let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config).unwrap();

    let token = ping_sender.send_to(Ipv4Addr::new(127, 0, 0, 1)).unwrap();

    let ping_response = ping_receiver.receive(token);

    assert!(ping_response.is_ok());
    assert!(matches!(ping_response.unwrap(), PingReceive::Data(_)));
}
