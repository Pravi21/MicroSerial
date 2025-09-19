#![cfg(unix)]

use std::time::{Duration, Instant};

use microserial_gui::core::SerialConfig;
use microserial_gui::session::{SerialSession, SessionMessage};
use nix::pty::{PtyMaster, openpty, ptsname};
use nix::unistd::{read, write};

#[test]
fn loopback_via_pty() {
    let pty = openpty(None, None).expect("openpty");
    let master = unsafe { PtyMaster::from_owned_fd(pty.master) };
    let slave_path = unsafe { ptsname(&master).expect("ptsname") };

    let mut session = SerialSession::open(&slave_path, &SerialConfig::default()).expect("session");

    write(&master, b"ping").expect("write master");
    let start = Instant::now();
    let mut rx = Vec::new();
    while start.elapsed() < Duration::from_secs(1) {
        for msg in session.poll() {
            if let SessionMessage::Data(bytes) = msg {
                rx.extend(bytes);
            }
        }
        if !rx.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(rx, b"ping");

    session.write(b"pong").expect("write session");
    let mut buf = [0u8; 8];
    let read_bytes = read(&master, &mut buf).expect("read");
    assert_eq!(&buf[..read_bytes], b"pong");
}
