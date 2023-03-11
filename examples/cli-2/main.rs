use ping_fox::{ping_runner_v2::PingRunnerV2Config, ping_runner_v2::SocketType, PingOutputData};
use std::net::Ipv4Addr;
use std::sync::{Arc, Condvar, Mutex};
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

#[derive(Clone)]
struct StopCondition {
    condition: Arc<(Mutex<bool>, Condvar)>,
}
impl StopCondition {
    pub(crate) fn new() -> Self {
        Self {
            condition: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub(crate) fn set_should_stop(&self) {
        let (lock, cvar) = &*self.condition;
        let mut should_stop = lock.lock().unwrap();
        *should_stop = true;
        cvar.notify_all();
    }

    pub(crate) fn get_should_stop(&self) -> bool {
        let (lock, _) = &*self.condition;
        let should_stop = lock.lock().unwrap();
        *should_stop
    }

    pub(crate) fn wait_timeout(&self, timeout: Duration) -> bool {
        let (lock, cvar) = &*self.condition;
        let guard = lock.lock().unwrap();
        let (should_stop, _) = cvar.wait_timeout(guard, timeout).unwrap();
        *should_stop
    }
}

// TODO: rename this example to cli (from cli-2)
fn main() -> Result<(), GenericError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::trace!("start");

    let args: Args = argh::from_env();

    let mut addresses = Vec::<Ipv4Addr>::new();
    for address in args.addresses {
        addresses.push(address.parse::<Ipv4Addr>()?);
    }
    tracing::trace!("addresses.len() == {}", addresses.len());
    tracing::trace!("args.count == {}", args.count);

    let config = PingRunnerV2Config {
        ips: &addresses,
        timeout: Duration::from_secs(1),
        channel_size: 8,
        socket_type: SocketType::DGRAM,
    };

    // TODO: Other notes:
    // You could have a send configuration and a receive configuration object?
    // Rename to PingCoordinator ?

    let (mut ping_sender, mut ping_receiver) =
        ping_fox::create::<ping_fox::icmp::v4::DgramSocket>(&config)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(8);
    let stop_condition_1 = StopCondition::new();
    let stop_condition_2 = stop_condition_1.clone();

    let thrd2 = std::thread::spawn(move || loop {
        let ping_result = ping_sender.send_ping_to_each_address();
        match ping_result {
            Err(e) => {
                println!("ERROR Err(e): {:?}", e);
                break;
            }
            Ok(evidences) => {
                tx.send(evidences);
            }
        }
        let should_stop: bool = stop_condition_1.wait_timeout(Duration::from_secs(1));
        if should_stop {
            break;
        }
    });

    let mut i = 0;
    'outer: loop {
        let ping_sent_evidences = rx.recv()?;
        for evidence in ping_sent_evidences {
            let ping_output = ping_receiver.receive_ping(evidence);
            match ping_output {
                Ok(Some(output)) => {
                    let PingOutputData {
                        package_size,
                        ip_addr,
                        ttl,
                        sequence_number,
                        ping_duration,
                    } = output;
                    println!(
                        "{package_size} bytes from {ip_addr}: \
                            icmp_seq={sequence_number} ttl={ttl} \
                            time={ping_duration:?}",
                    );
                    i += 1;
                }
                Err(e) => {
                    println!("ERROR Err(e): {:?}", e);
                }
                _ => {
                    // TODO
                }
            }
            if i >= args.count {
                break 'outer;
            }
        }
    }
    stop_condition_2.set_should_stop();
    thrd2.join();

    Ok(())
}
