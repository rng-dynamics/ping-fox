use crate::ping_error::GenericError;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::sync::mpsc;

pub(crate) enum PSetSender<T: std::any::Any> {
    Sync(mpsc::SyncSender<T>),
    NonSync(mpsc::Sender<T>),
}

pub(crate) type PSetDataT = (Ipv4Addr, u16);

pub(crate) struct PSet {
    set: HashSet<PSetDataT>,
    // Holding a copy of the sender in order the receiver is not disconnected before we read from
    // the receiver.
    _tx: PSetSender<PSetDataT>,
    rx: mpsc::Receiver<PSetDataT>,
}

impl PSet {
    pub(crate) fn new(tx: PSetSender<PSetDataT>, rx: mpsc::Receiver<PSetDataT>) -> Self {
        Self {
            set: HashSet::new(),
            _tx: tx,
            rx,
        }
    }

    pub(crate) fn update(&mut self) -> Result<(), GenericError> {
        loop {
            match self.rx.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // nothing to do
                    println!("PSet::update(): Err(Empty)");
                    break;
                }
                Ok(p) => {
                    println!("PSet::update(): Ok(p)");
                    let success = self.set.insert(p);
                    if !success {
                        println!("log ERROR: could not insert into hash set");
                    }
                }
                Err(e) => {
                    println!("PSet::update(): Err(e)");
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn contains(&self, data: &PSetDataT) -> bool {
        self.set.contains(data)
    }

    pub(crate) fn remove(&mut self, data: &PSetDataT) {
        self.set.remove(data);
    }
}
