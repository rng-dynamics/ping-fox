use std::net::IpAddr;
use std::sync::mpsc;
use std::time::Duration;

pub enum PingOutput {
    Data(PingOutputData),
    End,
}

// TODO: rename PingResponseData ?
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PingOutputData {
    pub package_size: usize,
    pub ip_addr: IpAddr,
    pub ttl: u8,
    pub sequence_number: u16,
    pub ping_duration: Duration,
}

// pub(crate) type PingOutputSender = mpsc::SyncSender<PingOutput>;
// pub(crate) type PingOutputReceiver = mpsc::Receiver<PingOutput>;
//
// pub(crate) fn ping_output_channel(channel_size: usize) -> (PingOutputSender, PingOutputReceiver) {
//     mpsc::sync_channel(channel_size)
// }
