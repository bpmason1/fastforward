extern crate http;
// extern crate rayon;

mod proxy;

use http::{
    header::HeaderValue,
    Response
};
use std::io;
use std::net::SocketAddr;
use proxy::generic_proxy;


fn my_director(req: &mut http::Request<Vec<u8>>) -> Option<Response<Vec<u8>>> { 
   // set the variables
   let proxy_addr = HeaderValue::from_str("127.0.0.1:4000").unwrap();

   let req_headers = req.headers_mut();
   req_headers.remove(http::header::HOST);
   req_headers.insert(http::header::HOST, proxy_addr);

//    *req.uri_mut() = "/health".parse().unwrap();
   None
}

fn main() -> io::Result<()> {
    let listen_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("running on port :8080");
    generic_proxy(listen_addr, my_director);

    Ok(())
}
