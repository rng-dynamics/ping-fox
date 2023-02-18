use c_dgram_socket_api::IcmpData;

mod c_dgram_socket_api {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

struct CDgramSocket {
    // socket: std::net::UdpSocket,
    socket: std::ffi::c_int,
    buffer: *mut std::ffi::c_char,
    buffer_len: usize,
}

impl CDgramSocket {
    fn recv_from(&self) -> i32 {
        unsafe {
            let mut icmp_data: IcmpData = IcmpData {
                data_buffer: self.buffer.cast::<u8>(),
                data_buffer_size: self.buffer_len as u64,
                n_data_bytes_received: 0,
                ttl: 0,
                addr_str: [0u8; 46],
            };
            c_dgram_socket_api::recv_from(self.socket, std::ptr::addr_of_mut!(icmp_data))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::os::unix::io::IntoRawFd;

    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    const BUFFER_LEN: usize = 256;

    #[test]
    fn recv_from_2_succeeds() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

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
                &"127.0.0.1:0".parse::<SocketAddr>().unwrap().into(),
            )
            .unwrap();

        let mut buffer = [0i8; BUFFER_LEN];
        let ptr: *mut std::ffi::c_char = buffer.as_mut_ptr();

        let c_dgram_socket = CDgramSocket {
            socket: socket.into_raw_fd(),
            buffer: ptr,
            buffer_len: BUFFER_LEN,
        };

        let n_received = c_dgram_socket.recv_from();
        assert!(n_received > 0);
        println!("{:?}", n_received);
        println!("{:?}", buffer);
    }
}
