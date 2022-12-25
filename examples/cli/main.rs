use std::net::Ipv4Addr;
use std::time::Duration;

use ping_fox::PingOutput;
use ping_fox::PingService;
use ping_fox::PingServiceConfig;

fn main() -> Result<(), std::net::AddrParseError> {
    let mut addresses = Vec::<Ipv4Addr>::new();
    for arg in std::env::args().skip(1) {
        addresses.push(arg.parse::<Ipv4Addr>()?);
    }
    let count = addresses.len();

    let ping_config = PingServiceConfig {
        ips: &addresses,
        count: 1,
        interval: Duration::from_secs(1),
        channel_size: 8,
    };

    let ping_service = PingService::create(ping_config).unwrap();

    for _ in 0..count {
        match ping_service.next_ping_output() {
            Ok(ok) => {
                let PingOutput {
                    package_size: payload_size,
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

    Ok(())
}
