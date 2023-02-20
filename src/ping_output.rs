use crate::Ttl;
use std::net::IpAddr;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug)]
pub struct PingOutput {
    pub package_size: usize,
    pub ip_addr: IpAddr,
    pub ttl: Ttl,
    pub sequence_number: u16,
    pub ping_duration: Duration,
}

pub(crate) type PingOutputSender = mpsc::SyncSender<PingOutput>;
pub(crate) type PingOutputReceiver = mpsc::Receiver<PingOutput>;

pub(crate) fn ping_output_channel(channel_size: usize) -> (PingOutputSender, PingOutputReceiver) {
    mpsc::sync_channel(channel_size)
}
