use std::net::IpAddr;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug)]
pub struct PingOutput {
    pub payload_size: usize,
    pub ip_addr: IpAddr,
    pub sequence_number: u16,
    pub ping_duration: Duration,
}

pub(crate) type PingOutputSender = mpsc::SyncSender<PingOutput>;
pub(crate) type PingOutputReceiver = mpsc::Receiver<PingOutput>;

pub(crate) fn ping_output_channel() -> (PingOutputSender, PingOutputReceiver) {
    // TODO: config
    mpsc::sync_channel(1024)
}
