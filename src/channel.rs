use std::sync::mpsc;

use crate::InternalData;

#[derive(Clone)]
pub(crate) struct SyncSender(mpsc::SyncSender<InternalData>);

impl SyncSender {
    pub(crate) fn send(&self, data: InternalData) -> Result<(), mpsc::SendError<InternalData>> {
        self.0.send(data)
    }
}

pub(crate) struct Receiver(mpsc::SyncSender<InternalData>, mpsc::Receiver<InternalData>);

impl Receiver {
    pub(crate) fn try_receive(&self) -> Result<InternalData, mpsc::TryRecvError> {
        self.1.try_recv()
    }
}

pub(crate) fn create_sync_channel(channel_size: usize) -> (SyncSender, Receiver) {
    let (tx, rx) = mpsc::sync_channel::<InternalData>(channel_size);
    (SyncSender(tx.clone()), Receiver(tx, rx))
}
