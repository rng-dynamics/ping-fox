extern crate ping_rs;
use ping_rs::PingService;

fn main() {
    let mut service = PingService::default();
    for arg in std::env::args().skip(1) {
        service = service.add_host(&arg);
    }
    let pinger = service.run_thread();
    for result in pinger.receiver.recv() {
        match result {
            Ok((hostname, ip, duration)) => {
                println!("{} {} {:?}", hostname, ip, duration);
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
        }
    }
    let _ = pinger.shutdown();
}
