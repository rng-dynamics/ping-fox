use ping_fox::{ping_runner_v2::PingRunnerV2Config, ping_runner_v2::SocketType, PingOutputData};
use std::net::Ipv4Addr;
use std::sync::Arc;
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
/// ping - send ICMP ECHO_REQUEST to IP address
struct Args {
    #[argh(positional)]
    /// IP addresses
    address: String,
}

fn main() -> Result<(), GenericError> {
    // TODO: set logging level appropriately
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Args = argh::from_env();

    let addresses = vec![args.address.parse::<Ipv4Addr>()?];
    let timeout = Duration::from_secs(1);

    let config = PingRunnerV2Config {
        ips: &addresses,
        timeout,
        channel_size: 1,
        socket_type: SocketType::DGRAM,
    };

    let (mut ping_sender, mut ping_receiver) =
        ping_fox::create::<ping_fox::icmp::v4::DgramSocket>(&config)?;
    let mut tokens = ping_sender.send_ping_to_each_address()?;
    let token = tokens.pop().expect("logic error: vec empty");
    let ping_response = ping_receiver.receive_ping(token);
    let PingOutputData {
        package_size,
        ip_addr,
        ttl,
        sequence_number,
        ping_duration,
    } = ping_response?.unwrap();
    println!(
        "{package_size} bytes from {ip_addr}: icmp_seq={sequence_number} ttl={ttl} time={ping_duration:?}",
    );

    Ok(())
}
