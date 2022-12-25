use std::net::Ipv4Addr;
use std::time::Duration;

use more_asserts as ma;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use ping_fox::*;

#[test]
fn test_ping_multiple_net() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // example.com 93.184.216.34
    let ip_example_com = Ipv4Addr::new(93, 184, 216, 34);
    // iana.com 192.0.43.8
    let ip_iana_com = Ipv4Addr::new(192, 0, 43, 8);
    let config = PingServiceConfig { channel_size: 4 };

    let ping_service = PingService::create_and_run(
        &[ip_example_com, ip_iana_com],
        1,
        Duration::from_secs(1),
        config,
    )
    .unwrap();

    // we expect two values
    let frst = ping_service.next_ping_output().unwrap();
    let scnd = ping_service.next_ping_output().unwrap();

    drop(ping_service);

    let ip_1_match_1 = frst.ip_addr == ip_example_com;
    let ip_1_match_2 = frst.ip_addr == ip_iana_com;
    assert!(ip_1_match_1 || ip_1_match_2);
    ma::assert_gt!(frst.ping_duration, Duration::from_secs(0));

    let ip_2_match_1 = scnd.ip_addr == ip_example_com;
    let ip_2_match_2 = scnd.ip_addr == ip_iana_com;
    assert!(ip_2_match_1 || ip_2_match_2);
    ma::assert_gt!(scnd.ping_duration, Duration::from_secs(0));
}
