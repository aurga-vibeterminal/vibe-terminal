use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use tokio::net::UdpSocket;

use super::types::LocalAddressSnapshot;

pub(crate) async fn discover_local_addresses(interface: Option<&str>) -> LocalAddressSnapshot {
    let interface_snapshot = discover_interface_addresses(interface);

    LocalAddressSnapshot {
        ipv4: interface_snapshot.ipv4.or(discover_local_ipv4().await),
        ipv6: interface_snapshot.ipv6.or(discover_local_ipv6().await),
    }
}

fn discover_interface_addresses(interface: Option<&str>) -> LocalAddressSnapshot {
    let Ok(interfaces) = if_addrs::get_if_addrs() else {
        return LocalAddressSnapshot::default();
    };

    let mut snapshot = LocalAddressSnapshot::default();
    for iface in interfaces {
        if iface.is_loopback() {
            continue;
        }

        if let Some(name) = interface {
            if iface.name != name {
                continue;
            }
        }

        match iface.addr {
            if_addrs::IfAddr::V4(address)
                if snapshot.ipv4.is_none() && is_usable_ipv4(address.ip) =>
            {
                snapshot.ipv4 = Some(address.ip);
            }
            if_addrs::IfAddr::V6(address)
                if snapshot.ipv6.is_none() && is_usable_ipv6(address.ip) =>
            {
                snapshot.ipv6 = Some(address.ip);
            }
            _ => {}
        }

        if snapshot.ipv4.is_some() && snapshot.ipv6.is_some() {
            break;
        }
    }

    snapshot
}

fn is_usable_ipv4(address: Ipv4Addr) -> bool {
    !address.is_loopback() && !address.is_unspecified()
}

fn is_usable_ipv6(address: Ipv6Addr) -> bool {
    !address.is_loopback()
        && !address.is_unspecified()
        && !address.is_multicast()
        && !address.is_unique_local()
        && !address.is_unicast_link_local()
}

async fn discover_local_ipv4() -> Option<Ipv4Addr> {
    discover_local_ip(
        SocketAddr::from(([0, 0, 0, 0], 0)),
        SocketAddr::from(([8, 8, 8, 8], 80)),
    )
    .await
    .and_then(|ip| match ip {
        IpAddr::V4(ipv4) => Some(ipv4),
        IpAddr::V6(_) => None,
    })
}

async fn discover_local_ipv6() -> Option<Ipv6Addr> {
    discover_local_ip(
        SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 0)),
        SocketAddr::from(([0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888], 80)),
    )
    .await
    .and_then(|ip| match ip {
        IpAddr::V4(_) => None,
        IpAddr::V6(ipv6) => Some(ipv6),
    })
}

async fn discover_local_ip(bind_addr: SocketAddr, probe_addr: SocketAddr) -> Option<IpAddr> {
    let socket = UdpSocket::bind(bind_addr).await.ok()?;
    socket.connect(probe_addr).await.ok()?;

    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ipv4) if !ipv4.is_unspecified() => Some(IpAddr::V4(ipv4)),
        IpAddr::V6(ipv6) if !ipv6.is_unspecified() => Some(IpAddr::V6(ipv6)),
        _ => None,
    }
}
