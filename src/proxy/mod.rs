mod filters;

use filters::remove_hop_by_hop_headers;
use hyper::{Client, Server};
//use hyper::client::ResponseFuture;
use hyper::service::service_fn;
use hyper::rt::{self, Future};
use std::net::SocketAddr;


type Director = fn(&mut hyper::Request<hyper::Body>); // -> Option<ResponseFuture>;

pub fn generic_proxy(listen_addr: SocketAddr, director: Director) {
    let client_main = Client::new();

    let new_service = move || {
        let client = client_main.clone();

        service_fn(move |mut req| {
            *req.headers_mut() = remove_hop_by_hop_headers(req.headers());
            (director)(&mut req);
            // let resp = (director)(&mut req);
            // match resp {
            //     Some(r) => { r }
            //     None => { client.request(req) }
            // }
            client.request(req)
        })
        
    };

    let server = Server::bind(&listen_addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));


    rt::run(server);
}

pub fn simple_proxy(listen_addr: SocketAddr, proxy_addr: SocketAddr) {
    let client_main = Client::new();
    
    let new_service = move || {
        let client = client_main.clone();

        service_fn(move |mut req| {
            *req.headers_mut() = remove_hop_by_hop_headers(req.headers());
            let scheme = req.uri().scheme_str().unwrap();
            let uri_string = format!(
                "{}://{}/{}",
                scheme,
                proxy_addr,
                req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
            let uri = uri_string.parse().unwrap();
            *req.uri_mut() = uri;

            client.request(req)
        })
    };

    let server = Server::bind(&listen_addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    rt::run(server);
}

