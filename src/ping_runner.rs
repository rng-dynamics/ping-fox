use crate::event::{
    ping_receive_event_channel, ping_send_event_channel, ping_send_sync_event_channel,
    PingSentSyncEvent,
};
use crate::ping_output::{ping_output_channel, PingOutput, PingOutputReceiver};
use crate::GenericError;
use crate::IcmpV4;
use crate::PingDataBuffer;
use crate::PingError;
use crate::PingReceiver;
use crate::PingSender;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

pub type PingResult<T> = std::result::Result<T, GenericError>;

pub struct PingRunner {
    states: Vec<State>,

    sender_thread: Option<JoinHandle<()>>,
    sender_halt_tx: mpsc::Sender<()>,

    receiver_thread: Option<JoinHandle<()>>,
    receiver_halt_tx: mpsc::Sender<()>,

    ping_output_rx: PingOutputReceiver,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Running,
    Halted,
}

impl Drop for PingRunner {
    fn drop(&mut self) {
        if let Err(e) = self.halt() {
            tracing::error!("{:#?}", e);
        }
    }
}

pub enum SocketType {
    DGRAM,
    RAW,
}

#[allow(clippy::module_name_repetitions)]
pub struct PingRunnerConfig<'a> {
    pub ips: &'a [Ipv4Addr],
    pub count: u16,
    pub interval: Duration,
    pub channel_size: usize,
    pub socket_type: SocketType,
}

impl PingRunner {
    // Create and start ping runner.
    pub fn create(config: &PingRunnerConfig<'_>) -> PingResult<Self> {
        let socket_timeout = Duration::from_millis(2000); // TODO
        match config.socket_type {
            SocketType::DGRAM => {
                let socket = Arc::new(crate::icmp::v4::CDgramSocket::create(socket_timeout)?);
                Ok(Self::create_with_socket(config, socket))
            }
            SocketType::RAW => {
                let socket = Arc::new(crate::icmp::v4::RawSocket::create(socket_timeout)?);
                Ok(Self::create_with_socket(config, socket))
            }
        }
    }

    fn create_with_socket<S>(config: &PingRunnerConfig<'_>, socket: Arc<S>) -> Self
    where
        S: crate::icmp::v4::Socket + 'static,
    {
        let mut deque = VecDeque::<Ipv4Addr>::new();
        for ip in config.ips {
            deque.push_back(*ip);
        }

        let (send_sync_event_tx, send_sync_event_rx) =
            ping_send_sync_event_channel(config.channel_size);
        let (receive_event_tx, receive_event_rx) = ping_receive_event_channel(config.channel_size);
        let (send_event_tx, send_event_rx) = ping_send_event_channel(config.channel_size);
        let (ping_output_tx, ping_output_rx) = ping_output_channel(config.channel_size);

        let (sender_halt_tx, sender_halt_rx) = mpsc::channel::<()>();
        let (receiver_halt_tx, receiver_halt_rx) = mpsc::channel::<()>();

        let ping_data_buffer = PingDataBuffer::new(send_event_rx, receive_event_rx, ping_output_tx);

        let icmpv4 = std::sync::Arc::new(IcmpV4::create());

        let ping_sender = PingSender::new(icmpv4, socket.clone(), send_event_tx);
        let ping_receiver = PingReceiver::new(socket, receive_event_tx);

        let sender_thread = Self::start_sender_thread(
            ping_sender,
            sender_halt_rx,
            config.count,
            deque,
            send_sync_event_tx,
            config.interval,
        );

        let receiver_thread = Self::start_receiver_thread(
            ping_data_buffer,
            ping_receiver,
            receiver_halt_rx,
            send_sync_event_rx,
        );

        Self {
            states: vec![State::Running],
            sender_thread: Some(sender_thread),
            sender_halt_tx,
            receiver_thread: Some(receiver_thread),
            receiver_halt_tx,
            ping_output_rx,
        }
    }

    pub fn next_ping_output(&self) -> PingResult<PingOutput> {
        if !self.is_in_state(State::Running) {
            return Err(PingError {
                message: "cannot next_ping_output() when PingRunner is not in state Running"
                    .to_string(),
            }
            .into());
        }
        Ok(self.ping_output_rx.recv()?)
    }

    fn halt(&mut self) -> std::thread::Result<()> {
        if self.is_in_state(State::Halted) {
            return Ok(());
        }
        // mpsc::Sender::send() returns error only if mpsc::Receiver is closed.
        let maybe_err_1 = self.sender_halt_tx.send(());
        if let Err(e) = maybe_err_1 {
            tracing::warn!("could not send shutdown message to sender thread: {}", e);
        }
        let maybe_err_2 = self.receiver_halt_tx.send(());
        if let Err(e) = maybe_err_2 {
            tracing::warn!("could not send shutdown message to receiver thread: {}", e);
        }

        let join_result_1 = match self.sender_thread.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };
        let join_result_2 = match self.receiver_thread.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        };

        join_result_1?;
        join_result_2?;

        self.states.push(State::Halted);
        Ok(())
    }

    fn is_in_state(&self, state: State) -> bool {
        match self.states.last() {
            None => false,
            Some(self_state) => *self_state == state,
        }
    }

    fn start_receiver_thread<S>(
        mut ping_data_buffer: PingDataBuffer,
        ping_receiver: PingReceiver<S>,
        halt_rx: mpsc::Receiver<()>,
        ping_send_sync_event_rx: mpsc::Receiver<PingSentSyncEvent>,
    ) -> JoinHandle<()>
    where
        S: crate::icmp::v4::Socket + 'static,
    {
        std::thread::spawn(move || {
            'outer: loop {
                // (1) Wait for sync-event from PingSender.
                let ping_send_sync_event_recv = ping_send_sync_event_rx.recv();

                if let Err(e) = ping_send_sync_event_recv {
                    tracing::info!("mpsc::Receiver::recv() failed: {}", e);
                    break 'outer;
                }

                // (2) receive ping and update ping buffer
                let receive_result = ping_receiver.receive();
                if let Err(e) = receive_result {
                    tracing::error!("PingReceiver::receive() failed: {}", e);
                    break 'outer;
                }
                ping_data_buffer.update();

                // (4) check termination
                match halt_rx.try_recv() {
                    Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }
            }
        })
    }

    fn start_sender_thread<S>(
        ping_sender: PingSender<S>,
        halt_rx: mpsc::Receiver<()>,
        count: u16,
        ips: VecDeque<Ipv4Addr>,
        ping_send_sync_event_tx: mpsc::SyncSender<PingSentSyncEvent>,
        interval: Duration,
    ) -> JoinHandle<()>
    where
        S: crate::icmp::v4::Socket + 'static,
    {
        std::thread::spawn(move || {
            'outer: for sequence_number in 0..count {
                for ip in &ips {
                    if ping_sender.send_one(*ip, sequence_number).is_err() {
                        tracing::error!("PingSender::send_one() failed");
                        break 'outer;
                    }
                    // (2.2) Dispatch sync event.
                    if ping_send_sync_event_tx.send(PingSentSyncEvent).is_err() {
                        tracing::error!("mpsc::SyncSender::send() failed");
                        break 'outer;
                    }
                    tracing::trace!("PingSender published SYNC-Event");

                    // (3) Check termination.
                    match halt_rx.try_recv() {
                        Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break 'outer,
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    }
                }
                if sequence_number < count - 1 {
                    // (4) Sleep according to configuration
                    tracing::trace!("PingSender will sleep");
                    std::thread::sleep(interval);
                }
            }
            tracing::trace!("PingSender thread end");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    #[test]
    fn ping_localhost_with_datagram_socket_succeeds() {
        // let subscriber = FmtSubscriber::builder()
        //     .with_max_level(Level::TRACE)
        //     .finish();
        // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

        let ping_config = PingRunnerConfig {
            ips: &[Ipv4Addr::new(127, 0, 0, 1)],
            count: 1,
            interval: Duration::from_secs(1),
            channel_size: 4,
            socket_type: SocketType::DGRAM,
        };

        let ping_runner = PingRunner::create(&ping_config).unwrap();
        let ping_output = ping_runner.next_ping_output();
        println!("{:?}", ping_output);
        assert!(ping_output.is_ok());
    }

    #[test]
    fn halt_succeeds() {
        let ping_config = PingRunnerConfig {
            ips: &[Ipv4Addr::new(127, 0, 0, 1)],
            count: 1,
            interval: Duration::from_secs(1),
            channel_size: 4,
            socket_type: SocketType::DGRAM,
        };

        let mut ping_runner = PingRunner::create(&ping_config).unwrap();
        assert!(ping_runner.halt().is_ok());
    }
}
