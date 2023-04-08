use crate::details::icmp::v4::{SequenceNumber, Ttl};
use std::time::Instant;
use std::{net::IpAddr, sync::mpsc};

#[derive(PartialEq, Eq)]
pub(crate) struct PingSendRecord {
    pub payload_size: usize,
    pub ip_addr: IpAddr,
    pub sequence_number: SequenceNumber,
    pub send_time: Instant,
}
pub(crate) type PingSendRecordSender = mpsc::SyncSender<PingSendRecord>;
pub(crate) type PingSendRecordReceiver = mpsc::Receiver<PingSendRecord>;
pub(crate) fn ping_send_record_channel(channel_size: usize) -> (PingSendRecordSender, PingSendRecordReceiver) {
    mpsc::sync_channel::<PingSendRecord>(channel_size)
}

#[derive(PartialEq, Eq)]
pub(crate) enum PingReceiveRecord {
    Timeout,
    Data(PingReceiveRecordData),
}

#[derive(PartialEq, Eq)]
pub(crate) struct PingReceiveRecordData {
    pub package_size: usize,
    pub ip_addr: IpAddr,
    pub ttl: Ttl,
    pub sequence_number: SequenceNumber,
    pub receive_time: Instant,
}
