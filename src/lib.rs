use if_addrs::Interface;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6};
use thiserror::Error;
use tokio::net::UdpSocket;

#[derive(Debug, Error)]
pub enum NetworkingError {
    #[error("std::io::Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unable to find interface number for the given networking interface")]
    UnableToFindInterfaceNumber,

    #[error("Only IPv6 addresses are supported")]
    Ipv4NotSupported,
}

pub fn get_interface(ip_or_name: &str) -> Option<Interface> {
    // Use the first interface we find where the interface name (e.g. `en0` or IP
    // address matches the argument. Note that we don't do any canonicalization on the
    // input value; for IPv6, addresses should be provided in their full, uncompressed
    // format.
    non_loopback_interfaces()
        .into_iter()
        .filter(|iface| iface.addr.ip().is_ipv6())
        .find(|iface| iface.addr.ip().to_string() == ip_or_name || iface.name == ip_or_name)
}

/// Creates a pair of sockets for receiving and sending multicast messages on the given port with
/// the given  interface.
pub fn create_broadcast_sockets(
    interface: &Interface,
    broadcast_socket_addr: &SocketAddrV6,
) -> Result<(UdpSocket, UdpSocket), NetworkingError> {
    if matches!(interface.ip(), IpAddr::V4(_)) {
        return Err(NetworkingError::Ipv4NotSupported);
    }
    // IPv6:
    // - for listening to broadcasts, we have to join a multicast IP address.
    // - for sending broadcast messages, we have to explicitly tell the socket which
    //   network interface to use, otherwise it won't have a route to the multicast IP.

    let Some(interface_idx) = interface.index else {
        return Err(NetworkingError::UnableToFindInterfaceNumber);
    };

    let broadcast_in_sock = {
        let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
        sock.join_multicast_v6(broadcast_socket_addr.ip(), interface_idx)?;
        sock.set_nonblocking(true)?;
        sock.set_only_v6(true)?;
        sock.set_reuse_address(true)?;
        sock.set_reuse_port(true)?;
        sock.bind(&SockAddr::from(SocketAddr::new(
            Ipv6Addr::UNSPECIFIED.into(),
            broadcast_socket_addr.port(),
        )))?;

        UdpSocket::from_std(sock.into())?
    };
    let broadcast_out_sock = {
        let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
        sock.set_multicast_if_v6(interface_idx)?;
        sock.set_nonblocking(true)?;
        sock.set_reuse_address(true)?;
        sock.set_reuse_port(true)?;
        sock.bind(&SockAddr::from(SocketAddr::new(
            Ipv6Addr::UNSPECIFIED.into(),
            0,
        )))?;

        UdpSocket::from_std(sock.into())?
    };

    Ok((broadcast_in_sock, broadcast_out_sock))
}

/// Get all non-loopback interfaces for this host.
pub fn non_loopback_interfaces() -> Vec<Interface> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(|addr| !addr.is_loopback())
        .collect()
}
