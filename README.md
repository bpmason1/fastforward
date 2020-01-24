[![Build Status](https://travis-ci.org/bpmason1/fastforward.svg?branch=master)](https://travis-ci.org/bpmason1/fastforward)

# fastforward

Fastforward is library for writing reverse proxies in rust.

## usage - generic_proxy
To implement arbitrary logic for your proxy implemenet a request director function.
The request, after be passed into the director will be served.
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


// The director function mutates the incoming request before proxying the
// request.  In this example, the request URI is changed to the proxy URI.
// This mimics the functionality of the fastforward::simple_proxy function.
fn my_director(req: &mut http::Request<Vec<u8>>) -> Option<Response<Vec<u8>>> { 
   // set the variables
   let proxy_addr = HeaderValue::from_str("127.0.0.1:4000").unwrap();

   let req_headers = req.headers_mut();
   req_headers.remove(http::header::HOST);
   req_headers.insert(http::header::HOST, proxy_addr);

   None  // ignore the return type for this example
}

fn main() -> io::Result<()> {
    let listen_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    generic_proxy(listen_addr, my_director);

    Ok(())
}
```
