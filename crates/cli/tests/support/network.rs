//! Network observer: a local recording proxy (ST-04 specs RG-ST04-4.1; plan P-13). Frozen.
//! Cut mode refuses at the proxy; it is not a network cut (clients must read the variables).

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

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

type Seen = Arc<Mutex<Vec<Attempt>>>;

pub struct Observer {
    addr: SocketAddr,
    seen: Seen,
}

impl Observer {
    /// Listens on `127.0.0.1:0`, one thread per connection.
    pub fn start(mode: Mode) -> Observer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind proxy");
        let addr = listener.local_addr().expect("proxy addr");
        let seen = Seen::default();
        let shared = seen.clone();
        // ponytail: the accept thread lives until the test process exits.
        thread::spawn(move || {
            for conn in listener.incoming().flatten() {
                let seen = shared.clone();
                thread::spawn(move || serve(conn, mode, &seen));
            }
        });
        Observer { addr, seen }
    }

    /// Variables for every process under test: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` (both
    /// cases) at the proxy, `NO_PROXY`/`no_proxy` empty.
    pub fn env(&self) -> Vec<(OsString, OsString)> {
        let url = format!("http://{}", self.addr);
        let proxied = ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"]
            .into_iter()
            .flat_map(|k| [k.to_string(), k.to_lowercase()])
            .map(|k| (k.into(), url.clone().into()));
        let bypass = ["NO_PROXY", "no_proxy"].map(|k| (k.into(), OsString::new()));
        proxied.chain(bypass).collect()
    }

    /// Attempts recorded since the last call, in arrival order (one command per call).
    pub fn take(&self) -> Vec<Attempt> {
        std::mem::take(&mut *self.seen.lock().expect("observer lock"))
    }
}

/// Reads the request head, records its target, then refuses (cut) or tunnels (open).
fn serve(conn: TcpStream, mode: Mode, seen: &Mutex<Vec<Attempt>>) {
    let mut reader = BufReader::new(conn);
    let mut head = String::new();
    let _ = reader.read_line(&mut head);
    let Some((connect, attempt)) = target(&head) else {
        return;
    };
    // Recorded before replying: the client cannot finish, so `take` after it sees the attempt.
    seen.lock().expect("observer lock").push(attempt.clone());
    if mode == Mode::Cut {
        return;
    }
    while matches!(reader.read_line(&mut head), Ok(n) if n > 0) && !head.ends_with("\r\n\r\n") {}
    let Ok(mut up) = TcpStream::connect((attempt.host.as_str(), attempt.port)) else {
        return;
    };
    let early = reader.buffer().to_vec();
    let mut down = reader.into_inner();
    let sent = if connect {
        down.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
    } else {
        up.write_all(head.as_bytes())
    };
    if sent.and_then(|()| up.write_all(&early)).is_ok() {
        pipe(&down, &up);
    }
}

/// `(is CONNECT, target)` of a request line: `CONNECT host:port` or an absolute `http://` URI.
fn target(line: &str) -> Option<(bool, Attempt)> {
    let mut words = line.split_whitespace();
    let (method, uri) = (words.next()?, words.next()?);
    let (connect, authority, default) = if method == "CONNECT" {
        (true, uri, 443)
    } else {
        let (scheme, rest) = uri.split_once("://")?;
        let default = if scheme == "https" { 443 } else { 80 };
        (false, rest.split('/').next()?, default)
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) if !p.contains(']') => (h, p.parse().ok()?),
        _ => (authority, default),
    };
    let host = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_string();
    Some((connect, Attempt { host, port }))
}

/// Copies bytes both ways; each direction ends with a write shutdown of its destination.
fn pipe(a: &TcpStream, b: &TcpStream) {
    let copy = |mut from: TcpStream, mut to: TcpStream| {
        let _ = std::io::copy(&mut from, &mut to);
        let _ = to.shutdown(Shutdown::Write);
    };
    let clone = |s: &TcpStream| s.try_clone().expect("clone");
    let (a2, b2) = (clone(a), clone(b));
    let t = thread::spawn(move || copy(a2, b2));
    copy(clone(b), clone(a));
    let _ = t.join();
}
