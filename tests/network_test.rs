use std::net::Ipv4Addr;

extern crate ping_rs;
use ping_rs::*;

// #[test]
// fn test_ping_multiple_net() {
//     // example.com 93.184.216.34
//     let ip_example_com = Ipv4Addr::new(93, 184, 216, 34);
//     // iana.com 192.0.43.8
//     let ip_iana_com = Ipv4Addr::new(192, 0, 43, 8);
//
//     let config = Config::new(64);
//     println!("test_pint_multiplt_net: 1");
//     let ping = Ping::start(&config, &[ip_example_com, ip_iana_com], 1);
//
//     std::thread::sleep(std::time::Duration::from_secs(1));
//
//     println!("test_pint_multiplt_net: 2");
//     // we expect two values
//     let frst = ping.next_result().unwrap();
//     println!("test_pint_multiplt_net: 3");
//     let scnd = ping.next_result().unwrap();
//     println!("test_pint_multiplt_net: 4");
//     // assert!(ping.receiver.recv().is_err());
//
//     let _ = ping.halt();
//
//     println!("test_pint_multiplt_net: 5");
//
//     println!("ip_1 == {:?}", frst);
//     let ip_1_match_1 = frst.1 == ip_example_com;
//     let ip_1_match_2 = frst.1 == ip_iana_com;
//     assert!(ip_1_match_1 || ip_1_match_2);
//     // assert_gt!(dur_1, Duration::from_secs(0));
//
//     println!("ip_2 == {:?}", scnd);
//     let ip_2_match_1 = scnd.1 == ip_example_com;
//     let ip_2_match_2 = scnd.1 == ip_iana_com;
//     assert!(ip_2_match_1 || ip_2_match_2);
//     // assert_gt!(dur_2, Duration::from_secs(0));
// }
