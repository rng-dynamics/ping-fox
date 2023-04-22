<!--
Template: https://gist.github.com/PurpleBooth/109311bb0361f32d87a2
-->

# ping-fox
![GitHub Workflow Status (with branch)](https://img.shields.io/github/actions/workflow/status/rng-dynamics/ping-fox/rust.yml?branch=main)
[![Coveralls branch](https://img.shields.io/coverallsCoverage/github/rng-dynamics/ping-fox?branch=main)](https://coveralls.io/github/rng-dynamics/ping-fox)

A Rust ping (ICMP) library without the need for elevated privileges.

## Getting Started


In ping-fox a `PingSentToken` represents an evidence that a ping message has been sent.
Each call to `PingSender::send_to` returns a `PingSentToken` which can be used to call `PingReceiver::recieve`.
This makes sure that `PingSender::recieve` is never called without a previous call to `PingSender::send_to`.


The following illustrates the simple usage of ping-fox.

```
# Cargo.toml

[dependencies]
ping-fox = { git = "https://github.com/rng-dynamics/ping-fox.git" }

```

``` rust
// .rs file

use ping_fox::{PingFoxConfig, PingReceive, PingReceiveData, PingSentToken, SocketType};
use std::net::Ipv4Addr;
use std::time::Duration;

// ### Configure the library settings:
// - `socket_type` can be `SocketType::RAW` or `SocketType::DGRAM`.
// - Use `SocketType::DGRAM` to avoid the need for elevated privileges.
let config = PingFoxConfig {
    socket_type: SocketType::DGRAM,
    timeout: Duration::from_secs(1),
    channel_size: 1,
};

// ### Create a sender and receiver ends of ping-fox.
let (mut ping_sender, mut ping_receiver) = ping_fox::create(&config).unwrap();

// ### Call `PingSender::send_to`
let token: PingSentToken = ping_sender
    .send_to("127.0.0.1".parse::<Ipv4Addr>().unwrap())
    .unwrap();

// ### Use the PingSentToken to call `PingReceiver::receive`.
let ping_response = ping_receiver.receive(token).unwrap();

match ping_response {
    PingReceive::Data(PingReceiveData {
        package_size,
        ip_addr,
        ttl,
        sequence_number,
        ping_duration,
    }) => {
        println!(
            "{package_size} bytes from {ip_addr}: \
              icmp_seq={sequence_number} ttl={ttl} \
              time={ping_duration:?}",
        );
    }
    PingReceive::Timeout => {
        println!("timeout");
    }
};
```

## Examples

There are some examples in the [example folder](examples/).

## Running the tests

- `cargo run --lib` will run the unit tests.
- We can run unit and integration tests with `cargo test`, but it will need elevated privileges for some of the tests using a raw socket. If we do not have privileges, some tests will fail.

## Built With

- [pnet_packet](https://crates.io/crates/pnet_packet)
- [socket2](https://crates.io/crates/socket2)
- [recvmsg](https://man7.org/linux/man-pages/man2/recvmsg.2.html) in order to obtain the time to live (TTL) without elevated privileges.

## Contributing

Contributions are welcome. Please open an issue and we can discuss the specifics.

## License

This project is licensed under the BSD-3-Clause license - see the [LICENSE.md](LICENSE.md) file for details
