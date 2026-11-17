use std::sync::Arc;
use std::time::Duration;

use tokio::net::{TcpStream, lookup_host};

use crate::BoxFuture;
use crate::exec::errno_name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetReason {
    Dns,
    Timeout,
    Refused,
    Unreachable,
    Closed,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetResult {
    Ok,
    Failed { reason: NetReason, message: String },
}

pub type TcpProbe =
    Arc<dyn Fn(String, u16, Duration) -> BoxFuture<'static, NetResult> + Send + Sync>;

pub fn reach_timeout(connect: bool) -> Duration {
    Duration::from_millis(if connect { 5_000 } else { 3_000 })
}

fn secs(d: Duration) -> String {
    let s = d.as_secs_f64();
    if s.fract() == 0.0 { format!("{}", s as u64) } else { format!("{s}") }
}

pub async fn probe(host: String, port: u16, timeout: Duration) -> NetResult {
    let lookup = tokio::time::timeout(timeout, lookup_host((host.as_str(), port))).await;
    let address = match lookup {
        Err(_) => {
            return NetResult::Failed {
                reason: NetReason::Timeout,
                message: format!("looking up {host} took over {} s", secs(timeout)),
            };
        }
        Ok(Err(_)) => {
            return NetResult::Failed {
                reason: NetReason::Dns,
                message: format!("cannot look up {host}"),
            };
        }
        Ok(Ok(mut addrs)) => match addrs.next() {
            Some(a) => a,
            None => {
                return NetResult::Failed {
                    reason: NetReason::Dns,
                    message: format!("cannot look up {host}"),
                };
            }
        },
    };
    match tokio::time::timeout(timeout, TcpStream::connect(address)).await {
        Err(_) => NetResult::Failed {
            reason: NetReason::Timeout,
            message: format!("no answer from {host}:{port} within {} s", secs(timeout)),
        },
        Ok(Ok(_)) => NetResult::Ok,
        Ok(Err(e)) => {
            let (reason, what) = match e.raw_os_error() {
                Some(libc::ECONNREFUSED) => (NetReason::Refused, "refused the connection".into()),
                Some(libc::EHOSTUNREACH) | Some(libc::ENETUNREACH) => {
                    (NetReason::Unreachable, "is unreachable".into())
                }
                _ => (NetReason::Other, format!("failed ({})", errno_name(&e))),
            };
            NetResult::Failed { reason, message: format!("{host}:{port} {what}") }
        }
    }
}

pub fn tcp_probe() -> TcpProbe {
    Arc::new(|host, port, timeout| Box::pin(probe(host, port, timeout)))
}
