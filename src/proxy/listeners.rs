use net2::TcpBuilder;
use std::io;
use std::net::SocketAddr;
use tokio_core::reactor::Handle;
use tokio_core::net::TcpListener;

pub fn setup_listener(addr: SocketAddr, handle: &Handle) -> io::Result<TcpListener> {
    let listener = TcpBuilder::new_v4()?;
    // // listener.reuse_address(true)?;
    // // listener.reuse_port(true)?;
    let listener = listener.bind(&addr)?;
    let listener = listener.listen(128)?;
    let listener = TcpListener::from_listener(listener, &addr, &handle)?;

    Ok(listener)
}
