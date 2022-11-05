use std::time::Instant;
use std::{net::IpAddr, sync::mpsc};

#[derive(PartialEq, Eq)]
pub(crate) struct PingSendEvent {
    pub payload_size: usize,
    pub ip_addr: IpAddr,
    pub sequence_number: u16,
    pub send_time: Instant,
}

#[derive(PartialEq, Eq)]
pub(crate) struct PingReceiveEventData {
    pub packet_size: usize,
    pub ip_addr: IpAddr,
    pub sequence_number: u16,
    pub receive_time: Instant,
}

#[derive(PartialEq, Eq)]
pub(crate) enum PingReceiveEvent {
    Timeout,
    Data(PingReceiveEventData), // TODO:
}
pub(crate) type PingSendEventSender = mpsc::SyncSender<PingSendEvent>;
pub(crate) type PingSendEventReceiver = mpsc::Receiver<PingSendEvent>;

pub(crate) fn ping_send_event_channel() -> (PingSendEventSender, PingSendEventReceiver) {
    // TODO: config
    mpsc::sync_channel::<PingSendEvent>(1024)
}

pub(crate) type PingReceiveEventSender = mpsc::SyncSender<PingReceiveEvent>;
pub(crate) type PingReceiveEventReceiver = mpsc::Receiver<PingReceiveEvent>;

pub(crate) fn ping_receive_event_channel() -> (PingReceiveEventSender, PingReceiveEventReceiver) {
    // TODO: config
    mpsc::sync_channel::<PingReceiveEvent>(1024)
}
