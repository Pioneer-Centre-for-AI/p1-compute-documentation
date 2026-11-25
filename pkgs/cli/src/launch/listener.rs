use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use ring::rand::{SecureRandom, SystemRandom};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Mine,
    Nothing,
    Foreign,
}

fn answers(port: u16) -> bool {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    TcpStream::connect_timeout(&addr, Duration::from_secs(1)).is_ok()
}

pub fn random_bytes<const N: usize>() -> std::io::Result<[u8; N]> {
    let mut bytes = [0u8; N];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| std::io::Error::other("the system gave no random bytes"))?;
    Ok(bytes)
}

pub fn any_port() -> Option<u16> {
    let n = u16::from_le_bytes(random_bytes::<2>().ok()?);
    Some(20_000 + n % 30_000)
}

/// A port on 127.0.0.1 that a server can listen on. A port is free only
/// when it can be bound: one that an outgoing connection of the node uses
/// answers nothing and still cannot be listened on.
pub fn free_port() -> Option<u16> {
    for _ in 0..50 {
        let port = any_port()?;
        if !answers(port) && std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).is_ok() {
            return Some(port);
        }
    }
    None
}

pub fn listeners(table: &str, port: u16) -> Vec<u32> {
    let mut uids = Vec::new();
    for line in table.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let (Some(local), Some(state), Some(uid)) = (fields.get(1), fields.get(3), fields.get(7))
        else {
            continue;
        };
        let Some((_, hex)) = local.rsplit_once(':') else { continue };
        if *state != "0A" || u16::from_str_radix(hex, 16) != Ok(port) {
            continue;
        }
        // A row that cannot be read counts as someone else's.
        uids.push(uid.parse().unwrap_or(u32::MAX));
    }
    uids
}

fn table() -> Option<String> {
    let v4 = std::fs::read_to_string("/proc/net/tcp").ok()?;
    let v6 = std::fs::read_to_string("/proc/net/tcp6").unwrap_or_default();
    Some(format!("{v4}\n{v6}"))
}

/// The server's token is reported only for `Owner::Mine`. Another user who
/// takes the port first would otherwise receive the browser, and the token.
pub fn owner(port: u16, uid: u32, pretend: Option<u32>) -> Owner {
    if !answers(port) {
        return Owner::Nothing;
    }
    let Some(table) = table() else { return Owner::Foreign };
    let found = listeners(&table, port);
    let mine = |u: &u32| pretend.unwrap_or(*u) == uid;
    if !found.is_empty() && found.iter().all(mine) { Owner::Mine } else { Owner::Foreign }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 11 1 0000000000000000 100 0 0 10 0
   1: 00000000:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  2000        0 12 1 0000000000000000 100 0 0 10 0
   2: 0100007F:1F91 0100007F:C350 01 00000000:00000000 00:00000000 00000000  3000        0 13 1 0000000000000000 100 0 0 10 0
   3: 00000000000000000000000001000000:1F92 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 14 1 0000000000000000 100 0 0 10 0
";

    #[test]
    fn listeners_are_the_listening_sockets_on_the_port_with_their_owners() {
        assert_eq!(listeners(TABLE, 0x1f90), [1000, 2000]);
        assert_eq!(listeners(TABLE, 0x1f91), [] as [u32; 0]);
        assert_eq!(listeners(TABLE, 0x1f92), [1000]);
        assert_eq!(listeners("   9: 0100007F:1F90 0:0 0A a b c x", 0x1f90), [u32::MAX]);
    }

    #[test]
    fn a_port_is_mine_only_when_i_own_every_listener_on_it() {
        let server = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = server.local_addr().unwrap().port();
        let uid = nix::unistd::getuid().as_raw();
        if table().is_some() {
            assert_eq!(owner(port, uid, None), Owner::Mine);
            assert_eq!(owner(port, uid, Some(uid.wrapping_add(1))), Owner::Foreign);
        }
        drop(server);
        assert_ne!(owner(port, uid, None), Owner::Foreign);
        assert!(free_port().is_some_and(|p| (20_000..50_000).contains(&p)));
    }
}
