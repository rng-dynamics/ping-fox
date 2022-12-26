use std::net::Ipv4Addr;
use std::time::Duration;

use ping_fox::PingOutput;
use ping_fox::PingRunner;
use ping_fox::PingRunnerConfig;

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
#[derive(Debug)]
struct Error {
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Error")?;
        if !self.message.is_empty() {
            write!(f, ": {}", self.message)?;
        }
        Ok(())
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

fn main() -> Result<(), GenericError> {
    let mut addresses = Vec::<Ipv4Addr>::new();
    for arg in std::env::args().skip(1) {
        addresses.push(arg.parse::<Ipv4Addr>()?);
    }
    let count = addresses.len();

    let ping_config = PingRunnerConfig {
        ips: &addresses,
        count: 1,
        interval: Duration::from_secs(1),
        channel_size: 8,
    };

    let ping_runner = PingRunner::create(ping_config)?;

    for _ in 0..count {
        match ping_runner.next_ping_output() {
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
