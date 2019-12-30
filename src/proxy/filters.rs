use hyper::HeaderMap;
use hyper::header::{self, HeaderName};


pub fn remove_hop_by_hop_headers(headers: &mut HeaderMap) {
    //for value in headers.get_all(header::CONNECTION).iter() {
    //    println!("Connection: {:?}", value);
    //}

    headers.remove(header::CONNECTION);
    headers.remove(HeaderName::from_static("keep-alive"));
    headers.remove(header::PROXY_AUTHENTICATE);
    headers.remove(header::PROXY_AUTHORIZATION);
    headers.remove(header::TRANSFER_ENCODING);
    headers.remove(header::TRAILER);
    headers.remove(header::UPGRADE);
}

