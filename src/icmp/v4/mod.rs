pub(crate) mod icmpv4;
mod sequence_number;
mod socket;

pub(crate) use icmpv4::IcmpV4;
pub(crate) use sequence_number::SequenceNumber;
// TODO: should be pub(crate) ?
pub use socket::dgram_socket::DgramSocket;
// TODO: should be pub(crate) ?
pub use socket::raw_socket::RawSocket;
pub(crate) use socket::Socket;

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) use super::socket::tests::OnReceive;
    pub(crate) use super::socket::tests::OnSend;
    pub(crate) use super::socket::tests::SocketMock;
}
