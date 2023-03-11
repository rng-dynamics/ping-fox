use std::net::Ipv4Addr;
use std::time::Duration;

use more_asserts as ma;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use ping_fox::{PingFoxConfig, PingReceive};

#[test]
fn test_ping_multiple_net() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // example.com 93.184.216.34
    let ip_example_com = Ipv4Addr::new(93, 184, 216, 34);
    // iana.com 192.0.43.8
    let ip_iana_com = Ipv4Addr::new(192, 0, 43, 8);
    let timeout = Duration::from_secs(1);

    let config = PingFoxConfig { ips: &[ip_example_com, ip_iana_com], timeout, channel_size: 2 };

    let (mut ping_sender, mut ping_receiver) = ping_fox::create::<ping_fox::icmp::v4::DgramSocket>(&config).unwrap();
    let mut tokens = ping_sender.send_ping_to_each_address().unwrap();
    let token1 = tokens.pop().expect("PingSentToken missing");
    let token2 = tokens.pop().expect("PingSentToken missing");

    if let PingReceive::Data(receive_data_1) = ping_receiver.receive_ping(token1).unwrap() {
        let ip_1_match_1 = receive_data_1.ip_addr == ip_example_com;
        let ip_1_match_2 = receive_data_1.ip_addr == ip_iana_com;
        assert!(ip_1_match_1 || ip_1_match_2);
        ma::assert_gt!(receive_data_1.ping_duration, Duration::from_secs(0));
    } else {
        panic!("ping receiver did not return expected data");
    }

    if let PingReceive::Data(receive_data_2) = ping_receiver.receive_ping(token2).unwrap() {
        let ip_2_match_1 = receive_data_2.ip_addr == ip_example_com;
        let ip_2_match_2 = receive_data_2.ip_addr == ip_iana_com;
        assert!(ip_2_match_1 || ip_2_match_2);
        ma::assert_gt!(receive_data_2.ping_duration, Duration::from_secs(0));
    } else {
        panic!("ping receiver did not return expected data");
    }
}
