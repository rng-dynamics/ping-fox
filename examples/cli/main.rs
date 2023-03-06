use ping_fox::{PingOutput, PingOutputData, PingRunner, PingRunnerConfig, SocketType};
use std::net::Ipv4Addr;
use std::time::Duration;

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

#[derive(argh::FromArgs)]
/// ping - send ICMP ECHO_REQUEST to IP addresses
struct Args {
    #[argh(option, short = 'c', default = "std::u16::MAX")]
    /// stop after <count> sent ping messages
    count: u16,

    #[argh(positional)]
    /// IP addresses
    addresses: Vec<String>,
}

fn main() -> Result<(), GenericError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::ERROR)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Args = argh::from_env();

    let mut addresses = Vec::<Ipv4Addr>::new();
    for address in args.addresses {
        addresses.push(address.parse::<Ipv4Addr>()?);
    }

    let ping_config = PingRunnerConfig {
        ips: &addresses,
        count: args.count,
        interval: Duration::from_secs(1),
        channel_size: 8,
        socket_type: SocketType::DGRAM,
    };

    let ping_runner = PingRunner::create(&ping_config)?;

    loop {
        match ping_runner.next_ping_output() {
            Ok(PingOutput::Data(ok)) => {
                let PingOutputData {
                    package_size: payload_size,
                    ip_addr,
                    ttl,
                    sequence_number,
                    ping_duration,
                } = ok;
                println!(
                    "{payload_size} bytes from {ip_addr}: \
                        icmp_seq={sequence_number} ttl={ttl} \
                        time={ping_duration:?}",
                );
            }
            Ok(PingOutput::End) => {
                break;
            }
            Err(e) => {
                println!("ERROR Err(e): {:?}", e);
            }
        }
    }

    Ok(())
}
