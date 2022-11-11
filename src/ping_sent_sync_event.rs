use std::sync::mpsc;

pub(crate) struct PingSentSyncEvent;

pub(crate) type PingSentSyncEventSender = mpsc::SyncSender<PingSentSyncEvent>;
pub(crate) type PingSentSyncEventReceiver = mpsc::Receiver<PingSentSyncEvent>;

pub(crate) fn ping_send_sync_event_channel() -> (PingSentSyncEventSender, PingSentSyncEventReceiver)
{
    // TODO: config
    mpsc::sync_channel::<PingSentSyncEvent>(1024)
}
