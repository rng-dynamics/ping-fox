use std::net::Ipv4Addr;
use std::time::Duration;

use more_asserts as ma;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use ping_fox::{PingRunner, PingRunnerConfig};

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

    let ping_config = PingRunnerConfig {
        ips: &[ip_example_com, ip_iana_com],
        count: 1,
        interval: Duration::from_secs(1),
        channel_size: 4,
    };

    let ping_runner = PingRunner::create(&ping_config).unwrap();

    // we expect two values
    let frst = ping_runner.next_ping_output().unwrap();
    let scnd = ping_runner.next_ping_output().unwrap();

    drop(ping_runner);

    let ip_1_match_1 = frst.ip_addr == ip_example_com;
    let ip_1_match_2 = frst.ip_addr == ip_iana_com;
    assert!(ip_1_match_1 || ip_1_match_2);
    ma::assert_gt!(frst.ping_duration, Duration::from_secs(0));

    let ip_2_match_1 = scnd.ip_addr == ip_example_com;
    let ip_2_match_2 = scnd.ip_addr == ip_iana_com;
    assert!(ip_2_match_1 || ip_2_match_2);
    ma::assert_gt!(scnd.ping_duration, Duration::from_secs(0));
}
