use ping_fox::{PingFoxConfig, PingReceive, PingReceiveData, SocketType};
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

#[derive(Clone)]
struct StopCondition {
    condition: Arc<(Mutex<bool>, Condvar)>,
}

impl StopCondition {
    pub(crate) fn new() -> Self {
        Self { condition: Arc::new((Mutex::new(false), Condvar::new())) }
    }

    pub(crate) fn set_should_stop(&self) {
        let (lock, cvar) = &*self.condition;
        let mut should_stop = lock.lock().unwrap();
        *should_stop = true;
        cvar.notify_all();
    }

    #[allow(dead_code)]
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
        .with_max_level(tracing::Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Args = argh::from_env();

    let mut addresses = Vec::<Ipv4Addr>::with_capacity(args.addresses.len());
    for address in args.addresses {
        addresses.push(address.parse::<Ipv4Addr>()?);
    }

    let config =
        PingFoxConfig { ips: &addresses, timeout: Duration::from_secs(1), channel_size: 8, socket_type: SocketType::DGRAM };

    let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(8);
    let stop_condition_1 = StopCondition::new();
    let stop_condition_2 = stop_condition_1.clone();

    let thrd2 = std::thread::spawn(move || loop {
        let ping_result = ping_sender.send_ping_to_each_address();
        match ping_result {
            Err(e) => {
                println!("ERROR: {:?}", e);
                break;
            }
            Ok(tokens) => {
                let send_tokens_result = tx.send(tokens);
                if let Err(e) = send_tokens_result {
                    println!("ERROR: {:?}", e);
                }
            }
        }
        let should_stop: bool = stop_condition_1.wait_timeout(Duration::from_secs(1));
        if should_stop {
            break;
        }
    });

    let mut i = 0;
    'outer: loop {
        let ping_sent_tokens = rx.recv()?;
        for token in ping_sent_tokens {
            let ping_output = ping_receiver.receive_ping(token);
            match ping_output {
                Ok(PingReceive::Data(PingReceiveData { package_size, ip_addr, ttl, sequence_number, ping_duration })) => {
                    println!(
                        "{package_size} bytes from {ip_addr}: icmp_seq={sequence_number} ttl={ttl} time={ping_duration:?}",
                    );
                    i += 1;
                }
                Ok(PingReceive::Timeout) => {
                    println!("receive timed out");
                }
                Err(e) => {
                    println!("ERROR: {:?}", e);
                }
            }
            if i >= args.count {
                break 'outer;
            }
        }
    }
    stop_condition_2.set_should_stop();
    let join_result = thrd2.join();
    if let Err(e) = join_result {
        println!("ERROR: {:?}", e);
    }

    Ok(())
}
