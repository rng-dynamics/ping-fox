use crate::event::PingSendEvent;
use crate::event::PingSendEventReceiver;
use crate::icmp::v4::SequenceNumber;
use crate::ping_error::PingError;
use crate::PingReceiveResultData;
use crate::{PingReceiveData, PingResult};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

pub(crate) struct PingDataBuffer {
    ping_send_event_rx: PingSendEventReceiver,

    send_events: HashMap<(SequenceNumber, IpAddr), (usize, Instant)>,
}

impl PingDataBuffer {
    pub(crate) fn new(ping_send_event_rx: PingSendEventReceiver) -> Self {
        Self {
            ping_send_event_rx,
            send_events: HashMap::new(),
        }
    }

    pub(crate) fn process_send_events(&mut self) -> usize {
        let mut n_send_events: usize = 0;
        while let Ok(send_event) = self.ping_send_event_rx.try_recv() {
            let PingSendEvent {
                payload_size,
                ip_addr,
                sequence_number,
                send_time,
            } = send_event;
            self.send_events
                .insert((sequence_number, ip_addr), (payload_size, send_time));
            n_send_events += 1;
        }
        n_send_events
    }

    pub(crate) fn process_receive_event(
        &mut self,
        data: &PingReceiveData,
    ) -> PingResult<PingReceiveResultData> {
        let PingReceiveData {
            package_size,
            ip_addr,
            ttl,
            sequence_number,
            receive_time,
        } = *data;
        match self.send_events.get(&(sequence_number, ip_addr)) {
            None => Err(PingError {
                message: "could not find matching data in send-events buffer".to_owned(),
            }
            .into()),
            Some(&(_payload_size, send_time)) => {
                self.send_events.remove(&(sequence_number, ip_addr));
                Ok(PingReceiveResultData {
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
