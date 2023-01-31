mod c_dgram_socket_api {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use c_dgram_socket_api::RecvData;

struct CDgramSocket {
    // socket: std::net::UdpSocket,
    socket: std::ffi::c_int,
    buffer: *mut std::ffi::c_char,
    buffer_len: usize,
}

impl CDgramSocket {
    fn recv_from(&self) -> RecvData {
        unsafe { c_dgram_socket_api::recv_from(self.socket, self.buffer, self.buffer_len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::os::unix::io::IntoRawFd;

    const BUFFER_LEN: usize = 256;

    #[test]
    fn recv_from_succeeds() {
        let icmpv4 = crate::IcmpV4::create();
        let package = icmpv4.new_icmpv4_package(0).unwrap();

        let socket: socket2::Socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::ICMPV4),
        )
        .unwrap();

        socket.set_ttl(128).unwrap();

        socket
            .send_to(
                pnet_packet::Packet::packet(&package),
                // &"127.0.0.1:7".parse::<SocketAddr>().unwrap().into(),
                // &"8.8.8.8:7".parse::<SocketAddr>().unwrap().into(),
                &"93.184.216.34:7".parse::<SocketAddr>().unwrap().into(),
            )
            .unwrap();

        let mut buffer = [0i8; BUFFER_LEN];
        let ptr: *mut std::ffi::c_char = buffer.as_mut_ptr();

        let c_dgram_socket = CDgramSocket {
            socket: socket.into_raw_fd(),
            buffer: ptr,
            buffer_len: BUFFER_LEN,
        };

        let recv_data = c_dgram_socket.recv_from();
        println!("{:?}", recv_data);
        let addr_str: String =
            String::from_utf8(recv_data.addr_str.iter().map(|&c| c as u8).collect()).unwrap();
        println!(
            "{:?}, {:?}, {:?}",
            recv_data.bytes_received, addr_str, recv_data.ttl
        );
        // println!("{:?}", buffer);
    }
}
