use crate::event::{
    PingReceiveEvent, PingReceiveEventReceiver, PingSendEvent, PingSendEventReceiver,
};
use crate::icmp::v4::SequenceNumber;
use crate::ping_output::{PingOutput, PingOutputData, PingOutputSender};
use crate::PingReceiveData;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

pub(crate) struct PingDataBuffer {
    ping_send_event_rx: PingSendEventReceiver,
    ping_receive_event_rx: PingReceiveEventReceiver,

    ping_output_tx: PingOutputSender,
    send_events: HashMap<(SequenceNumber, IpAddr), (usize, Instant)>,

    n_receive_events: usize,
}

impl PingDataBuffer {
    pub(crate) fn new(
        ping_send_event_rx: PingSendEventReceiver,
        ping_receive_event_rx: PingReceiveEventReceiver,
        ping_output_tx: PingOutputSender,
    ) -> Self {
        Self {
            ping_send_event_rx,
            ping_receive_event_rx,
            ping_output_tx,
            send_events: HashMap::new(),
            n_receive_events: 0,
        }
    }

    /// Return The number of successfully processed receive events
    pub(crate) fn update(&mut self) {
        self.process_send_events();
        self.process_receive_events();
    }

    pub(crate) fn get_num_of_receive_events(&self) -> usize {
        self.n_receive_events
    }

    fn process_send_events(&mut self) {
        while let Ok(send_event) = self.ping_send_event_rx.try_recv() {
            let PingSendEvent {
                payload_size,
                ip_addr,
                sequence_number,
                send_time,
            } = send_event;
            self.send_events
                .insert((sequence_number, ip_addr), (payload_size, send_time));
        }
    }

    fn process_receive_events(&mut self) {
        while let Ok(ping_receive_event) = self.ping_receive_event_rx.try_recv() {
            match ping_receive_event {
                PingReceiveEvent::Data(receive_data) => {
                    let PingReceiveData {
                        package_size,
                        ip_addr,
                        ttl,
                        sequence_number,
                        receive_time,
                    } = receive_data;
                    match self.send_events.get(&(sequence_number, ip_addr)) {
                        None => {
                            tracing::error!("could not find matching data in send-events buffer");
                            // TODO
                        }
                        Some(&(_payload_size, send_time)) => {
                            let send_result =
                                self.ping_output_tx.send(PingOutput::Data(PingOutputData {
                                    package_size,
                                    ip_addr,
                                    ttl: ttl.into(),
                                    sequence_number: sequence_number.0,
                                    ping_duration: receive_time - send_time,
                                }));
                            if let Err(e) = send_result {
                                tracing::error!("failed to send on PingOutput channel: {}", e);
                            } else {
                                self.n_receive_events += 1;
                            }
                            self.send_events.remove(&(sequence_number, ip_addr));
                        }
                    }
                }
                PingReceiveEvent::Timeout => {
                    tracing::warn!("timeout");
                    // TODO
                }
            }
        }
    }
}
