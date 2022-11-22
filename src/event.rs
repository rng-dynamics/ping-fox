use std::time::Instant;
use std::{net::IpAddr, sync::mpsc};

pub(crate) struct PingSentSyncEvent;
pub(crate) type PingSentSyncEventSender = mpsc::SyncSender<PingSentSyncEvent>;
pub(crate) type PingSentSyncEventReceiver = mpsc::Receiver<PingSentSyncEvent>;
pub(crate) fn ping_send_sync_event_channel(
    channel_size: usize,
) -> (PingSentSyncEventSender, PingSentSyncEventReceiver) {
    mpsc::sync_channel::<PingSentSyncEvent>(channel_size)
}

#[derive(PartialEq, Eq)]
pub(crate) struct PingSendEvent {
    pub payload_size: usize,
    pub ip_addr: IpAddr,
    pub sequence_number: u16,
    pub send_time: Instant,
}
pub(crate) type PingSendEventSender = mpsc::SyncSender<PingSendEvent>;
pub(crate) type PingSendEventReceiver = mpsc::Receiver<PingSendEvent>;
pub(crate) fn ping_send_event_channel(
    channel_size: usize,
) -> (PingSendEventSender, PingSendEventReceiver) {
    mpsc::sync_channel::<PingSendEvent>(channel_size)
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
    Data(PingReceiveEventData),
}
pub(crate) type PingReceiveEventSender = mpsc::SyncSender<PingReceiveEvent>;
pub(crate) type PingReceiveEventReceiver = mpsc::Receiver<PingReceiveEvent>;
pub(crate) fn ping_receive_event_channel(
    channel_size: usize,
) -> (PingReceiveEventSender, PingReceiveEventReceiver) {
    mpsc::sync_channel::<PingReceiveEvent>(channel_size)
}
