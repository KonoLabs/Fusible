//! Binds the dev server to the asked port, or to the next free one.

use std::io::ErrorKind;
use std::net::TcpListener;

use tiny_http::Server;

const PORT_TRIES: u16 = 20;

/// Binds on loopback from `port` upward and returns the server with the port it got.
///
/// A port in use moves on to the next one, up to `PORT_TRIES` ports. Any other bind
/// error stops at once.
pub(super) fn bind(port: u16) -> Result<(Server, u16), Box<dyn std::error::Error>> {
    let last = port.saturating_add(PORT_TRIES - 1);
    for candidate in port..=last {
        match TcpListener::bind(("127.0.0.1", candidate)) {
            Ok(listener) => {
                let server = Server::from_listener(listener, None).map_err(|e| e.to_string())?;
                return Ok((server, candidate));
            }
            Err(err) if err.kind() == ErrorKind::AddrInUse => continue,
            Err(err) => {
                return Err(
                    format!("cannot bind dev server to 127.0.0.1:{candidate}: {err}").into(),
                )
            }
        }
    }
    Err(format!("cannot bind dev server: ports {port} to {last} are all in use").into())
}
