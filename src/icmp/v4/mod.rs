mod icmpv4;
mod socket;

pub(crate) use icmpv4::IcmpV4;
pub(crate) use socket::default_timeout;
pub(crate) use socket::dgram_socket::CDgramSocket;
pub(crate) use socket::raw_socket::RawSocket;
pub(crate) use socket::Socket;

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) use super::socket::tests::OnReceive;
    pub(crate) use super::socket::tests::OnSend;
    pub(crate) use super::socket::tests::SocketMock;
}
