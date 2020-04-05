[![Build Status](https://travis-ci.org/bpmason1/fastforward.svg?branch=master)](https://travis-ci.org/bpmason1/fastforward)

# fastforward

Fastforward is library for writing reverse proxies in rust.


## usage - simple_proxy
To implement a proxy that just receives traffic on `listen_addr`
and forwards the it to `proxy_addr` use the `simple_proxy` function.
```
extern crate fastforward;
extern crate http;

use std::net::SocketAddr;
use proxy::simple_proxy;

fn main() {
    let listen_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let proxy_addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();

    println!("running on port :8080");
    simple_proxy(listen_addr, proxy_addr);
}
```

## usage - generic_proxy
To implement arbitrary logic for your proxy write a request director function.
The request, after be passed into the director, will be proxied to the address
specified by the "Host" header on the request.
```
extern crate fastforward;
extern crate http;

use http::{
    header::HeaderValue,
    Response
};
use std::io;
use std::net::SocketAddr;
use fastforward::generic_proxy;


// The director function mutates the incoming request before proxying it.
// In this example, the request URI is changed to the proxy URI.
// This examle mimics the functionality of the `simple_proxy` function.
fn my_director(req: &mut http::Request<Vec<u8>>) -> Option<Response<Vec<u8>>> { 
   // set the variables
   let proxy_addr = HeaderValue::from_str("127.0.0.1:4000").unwrap();

   let req_headers = req.headers_mut();
   req_headers.remove(http::header::HOST);
   req_headers.insert(http::header::HOST, proxy_addr);

   None  // ignore the return type for this example
}

fn main() {
    let listen_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    generic_proxy(listen_addr, my_director);
}
```

