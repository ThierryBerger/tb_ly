use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_PORT: u16 = 5888;
/// 0 means that the OS will assign any available port
pub const CLIENT_PORT: u16 = 0;

#[cfg(feature = "local")]
pub const GAME_SERVER_ADDR: &str = "127.0.0.1:5888";
/// Full URL of game server with reachable exposed port.
///
/// Behind fly.io, the port may not be the internal port.
#[cfg(not(feature = "local"))]
pub const GAME_SERVER_ADDR: &str = "213.188.212.111:5888";

#[cfg(feature = "local")]
pub const AUTH_SERVER_ADDR: &str = "0.0.0.0:4100";
/// Full URL of auth server with reachable exposed port.
///
/// Behind fly.io, the port may not be the internal port.
#[cfg(not(feature = "local"))]
pub const AUTH_SERVER_ADDR: &str = "213.188.212.111:4100";

pub const SHARED_SETTINGS: SharedSettings = SharedSettings { protocol_id: 0 };

pub const SEND_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Copy, Clone, Debug)]
pub struct SharedSettings {
    /// An id to identify the protocol version
    pub protocol_id: u64,
}
