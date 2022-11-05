use crate::ping_error::GenericError;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::mpsc;

// use crate::Receiver;
// 
// pub(crate) enum PSetSender<T: std::any::Any> {
//     Sync(mpsc::SyncSender<T>),
//     NonSync(mpsc::Sender<T>),
// }
// 
// pub(crate) type PSetDataT = (IpAddr, u16);
// 
// pub(crate) struct PSet {
//     set: HashSet<PSetDataT>,
//     receiver: Receiver,
// }
// 
// impl PSet {
//     pub(crate) fn new(receiver: Receiver) -> Self {
//         Self {
//             set: HashSet::new(),
//             receiver,
//         }
//     }
// 
//     pub(crate) fn update(&mut self) -> Result<(), GenericError> {
//         loop {
//             match self.receiver.try_receive() {
//                 Err(std::sync::mpsc::TryRecvError::Empty) => {
//                     // nothing to do
//                     println!("PSet::update(): Err(Empty)");
//                     break;
//                 }
//                 Ok((payload_size, ip_addr, sequence_number, send_time)) => {
//                     println!("PSet::update(): Ok(p)");
//                     let success = self.set.insert((ip_addr, sequence_number));
//                     if !success {
//                         println!("log ERROR: could not insert into hash set");
//                     }
//                 }
//                 Err(e) => {
//                     println!("PSet::update(): Err(e)");
//                     return Err(e.into());
//                 }
//             }
//         }
//         Ok(())
//     }
// 
//     pub(crate) fn contains(&self, data: &PSetDataT) -> bool {
//         self.set.contains(data)
//     }
// 
//     pub(crate) fn remove(&mut self, data: &PSetDataT) {
//         self.set.remove(data);
//     }
// }
