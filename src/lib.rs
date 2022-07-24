#![warn(rust_2018_idioms)]

#[cfg(test)]
#[macro_use]
extern crate more_asserts;

use std::collections::VecDeque;
use std::net::IpAddr;
use std::time::Duration;

mod icmpv4;
mod ping_error;
mod utils;

pub use ping_error::PingError;

pub type PingResult<T> = std::result::Result<T, PingError>;

pub struct PingEntity {
    pub receiver: std::sync::mpsc::Receiver<PingResult<(String, IpAddr, Duration)>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    thread_shutdown: std::sync::mpsc::Sender<()>,
}

impl PingEntity {
    // Note: this function consumes self
    pub fn shutdown(mut self) -> std::thread::Result<()> {
        let _ = self.thread_shutdown.send(());
        match self.thread_handle.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        }
    }
}

impl Drop for PingEntity {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            panic!("you must call `shutdown` on PingerThread to clean it up");
        }
    }
}

pub struct PingService {
    hostnames: VecDeque<String>,
}

impl Default for PingService {
    fn default() -> Self {
        PingService {
            hostnames: VecDeque::<String>::new(),
        }
    }
}

impl PingService {
    pub fn new(hostnames: VecDeque<String>) -> PingService {
        PingService { hostnames }
    }

    pub fn add_host(mut self, host: &str) -> Self {
        self.hostnames.push_back(host.to_owned());
        self
    }

    pub fn run_thread(&mut self) -> PingEntity {
        let (icmp_chan_in, icmp_chan_out) = std::sync::mpsc::channel();
        let (shutdown_chan_in, shutdown_chan_out) = std::sync::mpsc::channel();

        let hostnames = self.hostnames.clone();

        let thread = std::thread::spawn(move || {
            for hostname in hostnames {
                let maybe_ip = utils::lookup_host_v4(&hostname);
                let ping_result = match maybe_ip {
                    Ok(ip) => PingService::dispatch(&ip),
                    Err(e) => Err(e),
                };
                let send_result = icmp_chan_in.send(ping_result);
                if send_result.is_err() {
                    break;
                }
                match shutdown_chan_out.try_recv() {
                    Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }
            }
        });

        PingEntity {
            receiver: icmp_chan_out,
            thread_shutdown: shutdown_chan_in,
            thread_handle: Some(thread),
        }
    }

    fn dispatch(ip: &std::net::IpAddr) -> PingResult<(String, IpAddr, Duration)> {
        match ip {
            // TODO: sequence number?
            IpAddr::V4(ipv4) => icmpv4::ping_one(42, ipv4),
            IpAddr::V6(_ipv6) => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_localhost() {
        let mut ping = PingService::default();
        ping = ping.add_host("localhost");

        let pinger_thread: PingEntity = ping.run_thread();

        let (hostname, ip, dur) = pinger_thread.receiver.recv().unwrap().unwrap();

        let _ = pinger_thread.shutdown();

        assert_eq!(hostname, "localhost");
        assert_eq!(ip, std::net::Ipv4Addr::new(127, 0, 0, 1));
        assert_gt!(dur, Duration::from_secs(0));
    }
}
