extern crate ping_rs;

use std::net::Ipv4Addr;

use ping_rs::Config;
use ping_rs::PingRunner;
// use ping_rs::PingResult;
use ping_rs::PingOutput;

fn main() -> Result<(), std::net::AddrParseError> {
    let mut addresses = Vec::<Ipv4Addr>::new();
    for arg in std::env::args().skip(1) {
        addresses.push(arg.parse::<Ipv4Addr>()?);
    }
    let config = Config::new(32);

    let ping = PingRunner::start(&config, &addresses, 1);

    match ping.next_ping_output() {
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

    let _ = ping.halt();

    Ok(())
}
