mod listeners;

// use http::{Request, Response};
// use hyper::Body;
use hyper::{Client, Server};
// use hyper::client::HttpConnector;
// use hyper::server::conn::Http;
use hyper::service::service_fn;
// use hyper::service::Service;
use hyper::rt::{self, Future};
use std::net::SocketAddr;


// use listeners::setup_listener;

type Director = fn(&mut hyper::Request<hyper::Body>);

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
            (director)(&mut req);
            client.request(req)
        })
        
    };

    let server = Server::bind(&listen_addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    // println!("Listening on http://{}", listen_addr);

    rt::run(server);
}

