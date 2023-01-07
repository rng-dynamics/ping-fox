use std::net::Ipv4Addr;
use std::time::Duration;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use ping_fox::{PingRunner, PingRunnerConfig, SocketType};

/*
* Note: Raw sockets work only with root privileges.
*/
#[test]
fn ping_localhost_with_raw_socket_succeeds() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let ping_config = PingRunnerConfig {
        ips: &[Ipv4Addr::new(127, 0, 0, 1)],
        count: 1,
        interval: Duration::from_secs(1),
        channel_size: 4,
        socket_type: SocketType::RAW,
    };

    let ping_runner = PingRunner::create(&ping_config).unwrap();
    let ping_output = ping_runner.next_ping_output();
    assert!(ping_output.is_ok());
}
