use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

use crate::event::*;
use crate::ping_output::*;

pub(crate) struct PingDataBuffer {
    ping_send_event_rx: PingSendEventReceiver,
    ping_receive_event_rx: PingReceiveEventReceiver,

    ping_output_tx: PingOutputSender,
    // ping_output_rx: mpsc::Receiver<PingOutput>,
    send_events: HashMap<(u16, IpAddr), (usize, Instant)>,
}

impl PingDataBuffer {
    pub(crate) fn new(
        ping_send_event_rx: PingSendEventReceiver,
        ping_receive_event_rx: PingReceiveEventReceiver,
        ping_output_tx: PingOutputSender,
    ) -> Self {
        // let (ping_output_tx, ping_output_rx) = mpsc::sync_channel(1024);
        Self {
            ping_send_event_rx,
            ping_receive_event_rx,
            ping_output_tx,
            // ping_output_rx,
            send_events: HashMap::new(),
        }
    }
    pub(crate) fn update(&mut self) {
        self.process_send_events();
        self.process_receive_events();
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
                    let PingReceiveEventData {
                        packet_size,
                        ip_addr,
                        sequence_number,
                        receive_time,
                    } = receive_data;
                    match self.send_events.get(&(sequence_number, ip_addr)) {
                        None => {
                            // TODO
                        }
                        Some(&(payload_size, send_time)) => {
                            let send_result = self.ping_output_tx.send(PingOutput {
                                payload_size,
                                ip_addr,
                                sequence_number,
                                ping_duration: receive_time - send_time,
                            });
                            if let Err(e) = send_result {
                                tracing::error!("failed to send on PingOutput channel: {}", e);
                            }
                            self.send_events.remove(&(sequence_number, ip_addr));
                        }
                    }
                }
                PingReceiveEvent::Timeout => {
                    // TODO
                }
            }
        }
    }
}
