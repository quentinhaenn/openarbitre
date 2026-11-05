//! Network observer: a local recording proxy (ST-04 specs RG-ST04-4.1; plan P-13). Frozen.
//! Cut mode refuses at the proxy; it is not a network cut (clients must read the variables).

use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Forward every connection.
    Open,
    /// Refuse every connection.
    Cut,
}

/// One connection attempt seen by the proxy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub host: String,
    pub port: u16,
}

pub struct Observer {
    mode: Mode,
}

impl Observer {
    /// Listens on `127.0.0.1:0`, one thread per connection.
    pub fn start(mode: Mode) -> Observer {
        todo!("T-7 {mode:?}")
    }

    /// Variables for every process under test: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` (both
    /// cases) at the proxy, `NO_PROXY`/`no_proxy` empty.
    pub fn env(&self) -> Vec<(OsString, OsString)> {
        todo!("T-7 {:?}", self.mode)
    }

    /// Attempts recorded since the last call, in arrival order (one command per call).
    pub fn take(&self) -> Vec<Attempt> {
        todo!("T-7")
    }
}
