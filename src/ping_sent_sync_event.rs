use std::sync::mpsc;

pub(crate) struct PingSentSyncEvent;

pub(crate) type PingSentSyncEventSender = mpsc::SyncSender<PingSentSyncEvent>;
pub(crate) type PingSentSyncEventReceiver = mpsc::Receiver<PingSentSyncEvent>;

pub(crate) fn ping_send_sync_event_channel() -> (PingSentSyncEventSender, PingSentSyncEventReceiver)
{
    // TODO: config
    mpsc::sync_channel::<PingSentSyncEvent>(1024)
}

// #[derive(Clone)]
// pub(crate) struct PingSentSyncEventSender(mpsc::SyncSender<PingSentSyncEvent>);
// pub(crate) struct PingSentSyncEventReceiver(mpsc::Receiver<PingSentSyncEvent>);
//
// pub(crate) fn ping_sent_sync_event_channel() -> (PingSentSyncEventSender, PingSentSyncEventReceiver) {
//     // TODO: config
//     let (tx, rx) = mpsc::sync_channel::<PingSentSyncEvent>(1024);
//     (PingSentSyncEventSender(tx), PingSentSyncEventReceiver(rx))
// }
//
// impl PingSentSyncEventSender {
//     pub(crate) fn send(&self, ping_sent_sync: PingSentSyncEvent) -> Result<(), mpsc::SendError<PingSentSyncEvent>> {
//         self.0.send(ping_sent_sync)
//     }
// }
//
//
// impl PingSentSyncEventReceiver {
//     pub(crate) fn recv(&self) -> Result<PingSentSyncEvent, mpsc::RecvError> {
//         self.0.recv()
//     }
//
//     pub(crate) fn try_recv(&self) -> Result<PingSentSyncEvent, mpsc::TryRecvError> {
//         self.0.try_recv()
//     }
// }
