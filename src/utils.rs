use crate::ping_error::PingError;
use std::net::IpAddr;
use std::result::Result;

pub fn lookup_host(hostname: &str) -> Result<IpAddr, PingError> {
    let ips: Vec<std::net::IpAddr> = dns_lookup::lookup_host(hostname)?;
    ips.into_iter().next().ok_or(PingError {
        message: "could not resolve hostname ".to_owned() + hostname,
        source: None,
    })
}

pub fn lookup_host_v4(hostname: &str) -> Result<IpAddr, PingError> {
    let ips: Vec<std::net::IpAddr> = dns_lookup::lookup_host(hostname)?;
    ips.into_iter()
        .find(|&e| matches!(e, IpAddr::V4(_)))
        .ok_or(PingError {
            message: "could not resolve hostname ".to_owned() + " to IPv4",
            source: None,
        })
}

pub fn lookup_host_v6(hostname: &str) -> Result<IpAddr, PingError> {
    let ips: Vec<std::net::IpAddr> = dns_lookup::lookup_host(hostname)?;
    ips.into_iter()
        .find(|&e| matches!(e, IpAddr::V6(_)))
        .ok_or(PingError {
            message: "could not resolve hostname ".to_owned() + " to IPv6",
            source: None,
        })
}

pub fn lookup_addr(ip: IpAddr) -> Result<String, PingError> {
    let hostname = dns_lookup::lookup_addr(&ip)?;
    Ok(hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::Ipv4Addr;

    #[test]
    fn test_lookup_addr() {
        let ip_127_0_0_1 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        let hostname = lookup_addr(ip_127_0_0_1).unwrap();

        assert_eq!(hostname, "localhost");
    }

    #[test]
    fn test_lookup_host() {
        let ip = lookup_host_v4("localhost").unwrap();

        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }
}
