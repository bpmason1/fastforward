# fastforward

Fastforward is library for writing reverse proxies in rust.

## usage - simple_proxy
To proxy all requests from address `listen_addr` to address `proxy_addr`:
```
extern crate fastforward;
use fastforward::simple_proxy;

fn main() {
    let listen_addr = "0.0.0.0:80";
    let proxy_addr = "127.0.0.1:2000";

    simple_proxy(listen_addr, proxy_addr) // blocks and proxies requests
}
```
