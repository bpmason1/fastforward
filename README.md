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

## usage - generic_proxy
To implement arbitrary logic for your proxy implemenet a request director function.
The request, after be passed into the director will be served.
```
extern crate fastforward;
extern crate hyper;

use fastforward::generic_proxy;
use hyper::{Body, Request};

// The director function mutates the incoming request before proxying the
// request.  In this example, the request URI is changed to the proxy URI.
// This mimics the functionality of the fastforward::simple_proxy function.
fn my_director(req: &mut hyper::Request<Body>) {

    // set the variables
    let proxy_addr = "127.0.0.1:2000";
    let scheme = req.uri().scheme_str().unwrap();
    let path_and_qs = req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");

    // create the string for the new URI
    let uri_string = format!("{}://{}/{}",
        scheme,
        proxy_ip,
        path_and_qs);
    let uri = uri_string.parse().unwrap();

    // replace the original request URI with the new URI
    *req.uri_mut() = uri;
}

fn main() {
    let listen_addr = "0.0.0.0:80";

    generic_proxy(listen_addr, my_director) // blocks and proxies requests
}
```
