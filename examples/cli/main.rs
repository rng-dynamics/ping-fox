extern crate ping_rs;

use std::net::Ipv4Addr;

use ping_rs::Config;
use ping_rs::Ping;
use ping_rs::PingResult;

fn main() -> Result<(), std::net::AddrParseError> {
    println!("cli/main.rs");

    let mut addresses = Vec::<Ipv4Addr>::new();
    for arg in std::env::args().skip(1) {
        addresses.push(arg.parse::<Ipv4Addr>()?);
    }
    let config = Config::new(64);

    let ping = Ping::create(&config, &addresses, 1);

    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("cli/main.rs # 1");

    match ping.receiver.try_recv() {
        Ok(Ok((n, ip, sn))) => {
            println!("Ok Ok {} {} {}", n, ip, sn);
        }
        Ok(Err(e)) => {
            println!("ERROR Ok(Err(e)): {:?}", e);
        }
        Err(e) => {
            println!("ERROR Err(e): {:?}", e);
        }
    }
    // std::thread::sleep(std::time::Duration::from_secs(1));

    println!("cli/main.rs # 2");

    let _ = ping.shutdown();

    Ok(())
}
