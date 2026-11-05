//! Observer self-test with a plain std client (ST-04 E-ST04-4; matrix US-4 rows 1-2).

mod support;

use std::io::{Read, Write};
use std::net::TcpStream;

use support::network::{Attempt, Mode, Observer};

/// Proxy address from the observer's `HTTPS_PROXY` (`http://host:port`).
fn proxy(o: &Observer) -> String {
    let (_, v) = o
        .env()
        .into_iter()
        .find(|(k, _)| k == "HTTPS_PROXY")
        .expect("HTTPS_PROXY set");
    v.into_string()
        .expect("UTF-8")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

/// Sends `CONNECT host:port`; returns the proxy's first reply line, empty if closed.
fn connect(o: &Observer, target: &str) -> String {
    let mut s = TcpStream::connect(proxy(o)).expect("proxy reachable");
    // One write: cut mode closes after the request line, so a later fragment would be reset.
    let request = format!("CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n");
    s.write_all(request.as_bytes()).expect("send");
    let mut buf = [0u8; 64];
    let n = s.read(&mut buf).unwrap_or(0);
    String::from_utf8_lossy(&buf[..n])
        .lines()
        .next()
        .unwrap_or("")
        .to_string()
}

#[test]
fn env_points_every_client_at_the_proxy() {
    let o = Observer::start(Mode::Cut);
    let keys: Vec<_> = o.env().into_iter().map(|(k, _)| k).collect();
    for k in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "NO_PROXY",
        "no_proxy",
    ] {
        assert!(keys.iter().any(|x| x == k), "{k}");
    }
}

#[test]
fn cut_mode_records_and_refuses() {
    // Matrix US-4 row 2.
    let o = Observer::start(Mode::Cut);
    let reply = connect(&o, "relay.example:443");
    assert!(!reply.contains(" 200"), "refused: {reply:?}");
    assert_eq!(
        o.take(),
        [Attempt {
            host: "relay.example".into(),
            port: 443
        }]
    );
    assert!(o.take().is_empty(), "take drains");
}

#[test]
fn open_mode_records_and_forwards() {
    // Matrix US-4 row 1: a local listener stands for the destination.
    let dest = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = dest.local_addr().expect("addr").port();
    let o = Observer::start(Mode::Open);
    let reply = connect(&o, &format!("127.0.0.1:{port}"));
    assert!(reply.contains(" 200"), "forwarded: {reply:?}");
    assert_eq!(
        o.take(),
        [Attempt {
            host: "127.0.0.1".into(),
            port
        }]
    );
}
