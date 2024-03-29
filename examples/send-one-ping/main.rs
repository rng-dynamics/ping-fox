use ping_fox::{PingFoxConfig, PingReceive, PingReceiveData, SocketType};
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
/// ping - send ICMP ECHO_REQUEST to IP address
struct Args {
    #[argh(positional)]
    /// IP addresses
    address: String,
}

fn main() -> Result<(), GenericError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Args = argh::from_env();

    let address = args.address.parse::<Ipv4Addr>()?;
    let timeout = Duration::from_secs(1);

    let config = PingFoxConfig { timeout, channel_size: 1, socket_type: SocketType::DGRAM };

    let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config)?;
    let token = ping_sender.send_to(address)?;
    let ping_response = ping_receiver.receive(token);
    if let PingReceive::Data(PingReceiveData { package_size, ip_addr, ttl, sequence_number, ping_duration }) = ping_response?
    {
        println!("{package_size} bytes from {ip_addr}: icmp_seq={sequence_number} ttl={ttl} time={ping_duration:?}",);
    }

    Ok(())
}
