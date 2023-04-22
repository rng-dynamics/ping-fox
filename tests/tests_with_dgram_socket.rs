use ping_fox::{PingFoxConfig, PingReceive, SocketType};
use std::net::Ipv4Addr;
use std::sync::Once;
use std::time::Duration;

use more_asserts as ma;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static SETUP: Once = Once::new();

fn setup() {
    SETUP.call_once(|| {
        let subscriber = FmtSubscriber::builder().with_max_level(Level::ERROR).finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    });
}

#[test]
fn test_ping_to_localhost_with_dgram_socket() {
    setup();

    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let timeout = Duration::from_secs(1);

    let config = PingFoxConfig { timeout, channel_size: 1, socket_type: SocketType::DGRAM };

    let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config).unwrap();
    let token = ping_sender.send_to(localhost).unwrap();

    if let PingReceive::Data(receive_data) = ping_receiver.receive(token).unwrap() {
        assert_eq!(localhost, receive_data.ip_addr);
        ma::assert_gt!(receive_data.ping_duration, Duration::from_secs(0));
    } else {
        panic!("ping receiver did not return expected data");
    }
}

#[test]
fn test_ping_to_multiple_addresses_on_network_with_dgram_socket() {
    setup();

    // example.com 93.184.216.34
    let ip_example_com = Ipv4Addr::new(93, 184, 216, 34);
    // iana.com 192.0.43.8
    let ip_iana_com = Ipv4Addr::new(192, 0, 43, 8);
    let timeout = Duration::from_secs(1);

    let config = PingFoxConfig { timeout, channel_size: 2, socket_type: SocketType::DGRAM };

    let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config).unwrap();
    let token1 = ping_sender.send_to(ip_example_com).unwrap();
    let token2 = ping_sender.send_to(ip_iana_com).unwrap();

    if let PingReceive::Data(receive_data_1) = ping_receiver.receive(token1).unwrap() {
        let ip_1_match_1 = receive_data_1.ip_addr == ip_example_com;
        let ip_1_match_2 = receive_data_1.ip_addr == ip_iana_com;
        assert!(ip_1_match_1 || ip_1_match_2);
        ma::assert_gt!(receive_data_1.ping_duration, Duration::from_secs(0));
    } else {
        panic!("ping receiver did not return expected data");
    }

    if let PingReceive::Data(receive_data_2) = ping_receiver.receive(token2).unwrap() {
        let ip_2_match_1 = receive_data_2.ip_addr == ip_example_com;
        let ip_2_match_2 = receive_data_2.ip_addr == ip_iana_com;
        assert!(ip_2_match_1 || ip_2_match_2);
        ma::assert_gt!(receive_data_2.ping_duration, Duration::from_secs(0));
    } else {
        panic!("ping receiver did not return expected data");
    }
}
