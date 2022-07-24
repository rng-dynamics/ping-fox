use std::time::Duration;

#[macro_use]
extern crate more_asserts;

extern crate ping_rs;
use ping_rs::*;

#[test]
fn test_ping_multiple_net() {
    let mut pinger = PingService::default();
    pinger = pinger.add_host("example.com");
    pinger = pinger.add_host("iana.com");

    let pinger_thread = pinger.run_thread();

    // we expect two values
    let frst = pinger_thread.receiver.recv().unwrap();
    let scnd = pinger_thread.receiver.recv().unwrap();
    assert!(pinger_thread.receiver.recv().is_err());

    let _ = pinger_thread.shutdown();

    let (hostname_1, ip_1, dur_1) = frst.unwrap();
    assert_eq!(hostname_1, "93.184.216.34");
    assert_eq!(ip_1, std::net::Ipv4Addr::new(93, 184, 216, 34));
    assert_gt!(dur_1, Duration::from_secs(0));

    let (hostname_2, ip_2, dur_2) = scnd.unwrap();
    assert_eq!(hostname_2, "43-8.any.icann.org");
    assert_eq!(ip_2, std::net::Ipv4Addr::new(192, 0, 43, 8));
    assert_gt!(dur_2, Duration::from_secs(0));
}
