# Template: https://gist.github.com/PurpleBooth/109311bb0361f32d87a2

# ping-fox
![GitHub Workflow Status (with branch)](https://img.shields.io/github/actions/workflow/status/rng-dynamics/ping-fox/rust.yml?branch=main)
[![Coveralls branch](https://img.shields.io/coverallsCoverage/github/rng-dynamics/ping-fox?branch=main)](https://coveralls.io/github/rng-dynamics/ping-fox)

A Rust ping (ICMP) library without the need for elevated privileges.

## Getting Started

Simple usage:

```
# Cargo.toml

[dependencies]
ping-fox = { git = "https://github.com/rng-dynamics/ping-fox.git" }

```

``` rust
// .rs file

use ping_fox::{PingFoxConfig, PingReceive, PingReceiveData, SocketType};
use std::net::Ipv4Addr;
use std::time::Duration;

let addresses = vec!["127.0.0.1".parse::<Ipv4Addr>()?];
let timeout = Duration::from_secs(1);

// Use `SocketType::DGRAM` in order to avoid the need for elevated privileges.
let config = PingFoxConfig {
    ips: &addresses,
    timeout,
    channel_size: 1,
    socket_type: SocketType::DGRAM,
};

let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config)?;

// Sending pings returns a vector of tokens which we use in order to call `receive_ping`.
let mut tokens = ping_sender.send_ping_to_each_address()?;
let token = tokens.pop().expect("logic error: vec empty");

// We pass the token obtained from `send_pint_to_each_address` to the receive call.
let ping_response = ping_receiver.receive_ping(token)?;

match ping_response {
    PingReceive::Data(PingReceiveData {
        package_size,
        ip_addr,
        ttl,
        sequence_number,
        ping_duration,
    }) => {
        println!("{package_size} bytes from {ip_addr}: \
                  icmp_seq={sequence_number} ttl={ttl} \
                  time={ping_duration:?}",);
    }
    PingReceive::Timeout => {
        println!("timeout");
    }
};
```

## Examples

There are examples in the [example folder](examples/).

## Running the tests

- `cargo run --lib` will run the unit tests.
- We can run unit and integration tests with `cargo test`, but it will need elevated privileges for some of the tests using a raw socket. If we do not have privileges, some tests will fail.
- Y

## Built With

- [pnet_packet](https://crates.io/crates/pnet_packet)
- [socket2](https://crates.io/crates/socket2)
- [recvmsg](https://man7.org/linux/man-pages/man2/recvmsg.2.html) in order to obtain the time to live (TTL) without elevated privileges.

## Contributing

Contributions are welcome. Please open an issue and we can discuss the specifics.

## License

This project is licensed under the BSD-3-Clause license - see the [LICENSE.md](LICENSE.md) file for details
