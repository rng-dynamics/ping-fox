use crate::icmp::v4::SequenceNumber;
use crate::ping_error::PingError;
use crate::records::PingReceiveRecordData;
use crate::records::PingSendRecord;
use crate::records::PingSendRecordReceiver;
use crate::PingReceiveData;
use crate::PingResult;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

pub(crate) struct PingDataBuffer {
    ping_send_record_rx: PingSendRecordReceiver,

    send_records: HashMap<(SequenceNumber, IpAddr), (usize, Instant)>,
}

impl PingDataBuffer {
    pub(crate) fn new(ping_send_record_rx: PingSendRecordReceiver) -> Self {
        Self { ping_send_record_rx, send_records: HashMap::new() }
    }

    pub(crate) fn process_send_records(&mut self) -> usize {
        let mut n_send_records: usize = 0;
        while let Ok(send_record) = self.ping_send_record_rx.try_recv() {
            let PingSendRecord { payload_size, ip_addr, sequence_number, send_time } = send_record;
            self.send_records
                .insert((sequence_number, ip_addr), (payload_size, send_time));
            n_send_records += 1;
        }
        n_send_records
    }

    pub(crate) fn process_receive_record(&mut self, data: &PingReceiveRecordData) -> PingResult<PingReceiveData> {
        let PingReceiveRecordData { package_size, ip_addr, ttl, sequence_number, receive_time } = *data;
        match self.send_records.get(&(sequence_number, ip_addr)) {
            None => Err(PingError { message: "could not find matching data in send-records buffer".to_owned() }.into()),
            Some(&(_payload_size, send_time)) => {
                self.send_records.remove(&(sequence_number, ip_addr));
                Ok(PingReceiveData {
                    package_size,
                    ip_addr,
                    ttl: ttl.into(),
                    sequence_number: sequence_number.into(),
                    ping_duration: receive_time - send_time,
                })
            }
        }
    }
}
