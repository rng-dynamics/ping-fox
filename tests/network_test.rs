use std::net::Ipv4Addr;

extern crate ping_rs;
use ping_rs::*;

#[test]
fn test_ping_multiple_net() {
    // example.com 93.184.216.34
    let ip_example_com = Ipv4Addr::new(93, 184, 216, 34);
    // iana.com 192.0.43.8
    let ip_iana_com = Ipv4Addr::new(192, 0, 43, 8);

    let config = Config::new(64);
    println!("test_pint_multiplt_net: 1");
    let ping = Ping::create(&config, &[ip_example_com, ip_iana_com], 1);

    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("test_pint_multiplt_net: 2");
    // we expect two values
    let frst = ping.receiver.recv().unwrap();
    println!("test_pint_multiplt_net: 3");
    let scnd = ping.receiver.recv().unwrap();
    println!("test_pint_multiplt_net: 4");
    // assert!(ping.receiver.recv().is_err());

    let _ = ping.shutdown();

    println!("test_pint_multiplt_net: 5");

    let r_1 = frst.unwrap();
    println!("ip_1 == {:?}", r_1);
    let ip_1_match_1 = r_1.1 == ip_example_com;
    let ip_1_match_2 = r_1.1 == ip_iana_com;
    assert!(ip_1_match_1 || ip_1_match_2);
    // assert_gt!(dur_1, Duration::from_secs(0));

    let r_2 = scnd.unwrap();
    println!("ip_2 == {:?}", r_2);
    let ip_2_match_1 = r_2.1 == ip_example_com;
    let ip_2_match_2 = r_2.1 == ip_iana_com;
    assert!(ip_2_match_1 || ip_2_match_2);
    // assert_gt!(dur_2, Duration::from_secs(0));
}
