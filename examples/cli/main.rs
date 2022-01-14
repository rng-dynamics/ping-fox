extern crate kingping;
use kingping::Ping;

fn main() {
    let mut p = Ping::new();
    for arg in std::env::args().skip(1) {
        p.add_host(&arg);
    }
    let results = p.run();
    for result in results {
        match result {
            Ok((hostname, ip, duration)) => {
                println!("{} {} {:?}", hostname, ip, duration);
            }
            Err(e) => {
                println!("ERR: {:?}", e);
            }
        }
    }
}
