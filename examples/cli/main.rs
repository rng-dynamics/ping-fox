extern crate kingping;
use kingping::PingService;

fn main() {
    let mut p = PingService::default();
    for arg in std::env::args().skip(1) {
        p = p.add_host(&arg);
    }
    let pinger_thread = p.run_thread();
    for result in pinger_thread.receiver.recv() {
        match result {
            Ok((hostname, ip, duration)) => {
                println!("{} {} {:?}", hostname, ip, duration);
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
        }
    }
    let _ = pinger_thread.shutdown();
}
