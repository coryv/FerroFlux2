use std::net::{IpAddr, ToSocketAddrs};
use url::Url;

/// Validates that a given URL does not point to a private or reserved network address.
///
/// This function parses the URL, extracts the host, resolves it to an IP address,
/// and checks if that IP address falls within blocked ranges (loopback, private, link-local).
pub fn validate_url(url_str: &str) -> Result<(), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // We only validate http/https/ftp/ssh schemes where we expect network connections
    // If the scheme is not one of these, we might want to fail or just warn, but
    // for now we focus on the host validation.

    let host = url.host_str().ok_or("URL missing host")?;
    let port = url.port_or_known_default().ok_or("URL missing port")?;

    validate_host_port(host, port)
}

/// Validates that a host and port do not resolve to a private or reserved network address.
pub fn validate_host_port(host: &str, port: u16) -> Result<(), String> {
    // Resolve the host to IP addresses
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("Failed to resolve host '{}': {}", host, e))?;

    for addr in addrs {
        let ip = addr.ip();
        if is_blocked_ip(ip) {
            return Err(format!(
                "Host '{}' resolves to blocked IP address '{}'",
                host, ip
            ));
        }
    }

    Ok(())
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    // Block Loopback (127.0.0.0/8)
    if ip.is_loopback() {
        return true;
    }

    // Block Unspecified (0.0.0.0)
    if ip.is_unspecified() {
        return true;
    }

    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();

            // Block Private Networks
            // 10.0.0.0/8
            if octets[0] == 10 {
                return true;
            }

            // 172.16.0.0/12
            if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                return true;
            }

            // 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 {
                return true;
            }

            // Block Link-Local (169.254.0.0/16)
            if octets[0] == 169 && octets[1] == 254 {
                return true;
            }

            // Block Broadcast (255.255.255.255)
            if ipv4.is_broadcast() {
                return true;
            }
        }
        IpAddr::V6(ipv6) => {
            // Block Unique Local Address (fc00::/7) - roughly equivalent to private IPv4
            // Range is fc00::/7 (fc00... to fdff...)
            if (ipv6.segments()[0] & 0xfe00) == 0xfc00 {
                return true;
            }

            // Block Link-Local Unicast (fe80::/10)
            if (ipv6.segments()[0] & 0xffc0) == 0xfe80 {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_blocked_ip_v4() {
        assert!(is_blocked_ip("127.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("10.0.0.5".parse().unwrap()));
        assert!(is_blocked_ip("192.168.1.1".parse().unwrap()));
        assert!(is_blocked_ip("172.16.0.1".parse().unwrap()));
        assert!(is_blocked_ip("172.31.255.255".parse().unwrap()));
        assert!(is_blocked_ip("169.254.0.1".parse().unwrap()));

        // Allowed public IPs
        assert!(!is_blocked_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_blocked_ip("1.1.1.1".parse().unwrap()));
        assert!(!is_blocked_ip("172.32.0.1".parse().unwrap())); // Outside private range
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("http://google.com").is_ok());
        // We can't easily test "http://localhost" because it depends on DNS resolution of the machine running tests.
        // But we can test raw IPs in URLs.
        assert!(validate_url("http://127.0.0.1").is_err());
        assert!(validate_url("http://192.168.0.1:8080").is_err());
    }
}
