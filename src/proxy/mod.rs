// mod listeners;
mod filters;

// use http::{Request, Response};
// use hyper::Body;
use hyper::{Client, Server};
use hyper::client::ResponseFuture;
// use hyper::server::conn::Http;
use hyper::service::service_fn;
// use hyper::service::Service;
use hyper::rt::{self, Future};
use std::net::SocketAddr;

use filters::filter_request_headers;


// use listeners::setup_listener;

type Director = fn(&mut hyper::Request<hyper::Body>) -> Option<ResponseFuture>;

// struct Proxy {
//     pub client: Client<HttpConnector, Body>,
//     pub director: Director,
// }

// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

pub fn generic_proxy(listen_addr: SocketAddr, director: Director) {
    let client_main = Client::new();

    let new_service = move || {
        let client = client_main.clone();

        service_fn(move |mut req| {
            filter_request_headers(req.headers_mut());
            let resp = (director)(&mut req);
            match resp {
                Some(r) => { r }
                None => { client.request(req) }
            }
        })
        
    };

    let server = Server::bind(&listen_addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    // println!("Listening on http://{}", listen_addr);

    rt::run(server);
}

pub fn simple_proxy(listen_addr: SocketAddr, proxy_addr: SocketAddr) {
    let client_main = Client::new();
    
    let new_service = move || {
        let client = client_main.clone();

        service_fn(move |mut req| {
            filter_request_headers(req.headers_mut());
            let scheme = req.uri().scheme_str().unwrap();
            let uri_string = format!("{}://{}/{}",
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

    // println!("Listening on http://{}", listen_addr);

    rt::run(server);

}
