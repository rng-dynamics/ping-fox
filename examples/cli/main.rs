use std::net::Ipv4Addr;
use std::time::Duration;

use ping_rs::PingRs;
// use ping_rs::PingResult;
use ping_rs::PingOutput;

fn main() -> Result<(), std::net::AddrParseError> {
    let mut addresses = Vec::<Ipv4Addr>::new();
    for arg in std::env::args().skip(1) {
        addresses.push(arg.parse::<Ipv4Addr>()?);
    }
    let count = addresses.len();

    let mut ping_rs = PingRs::new(32);
    ping_rs.run(&addresses, 1, Duration::from_secs(1)).unwrap();

    for _ in 0..count {
        match ping_rs.next_ping_output() {
            Ok(ok) => {
                let PingOutput {
                    payload_size,
                    ip_addr,
                    sequence_number,
                    ping_duration,
                } = ok;
                println!(
                    "Ok {} {} {} {:#?}",
                    payload_size, ip_addr, sequence_number, ping_duration
                );
            }
            Err(e) => {
                println!("ERROR Err(e): {:?}", e);
            }
        }
    }

    let _ = ping_rs.halt();

    Ok(())
}
