mod listeners;

use futures::{Future, Stream};
use http::{Request, Response};
use hyper::body::Payload;
use hyper::{Body, Client, Server};
use hyper::client::HttpConnector;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::service::Service;
use hyper::rt; //::{self, Future};
use std::net::SocketAddr;
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Handle};

use listeners::setup_listener;

type Director = fn(&mut Request<()>);

struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub director: Director,
}

pub fn generic_proxy(listen_addr: SocketAddr, director: Director) {
    // let mut core = Core::new().unwrap();
    // let handle: Handle = core.handle();

    // let listener = setup_listener(listen_addr, &handle).expect("Failed to setup listener");
    // let clients = listener.incoming();

    // let srv = clients.for_each(move |(socket, _)| {
    //     _proxy(socket, &handle, director);
    //     Ok(())
    // });

    let client_main = Client::new();
    
    let new_service = move || {
        let client = client_main.clone();

        service_fn(move |mut req| {
            let uri_string = format!("http://{}/{}",
                "127.0.0.1:4000",
                req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
            let uri = uri_string.parse().unwrap();
            *req.uri_mut() = uri;
            client.request(req)
        })
        
    };

    let server = Server::bind(&listen_addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", listen_addr);
    // println!("Proxying on http://{}", out_addr);

    rt::run(server);
}

fn _proxy(socket: TcpStream, handle: &Handle, director: Director) {
    socket.set_nodelay(true).unwrap();
    let client_main = Client::new();

    // let client = Client::configure()
    //     // .connector(tm)
    //     .build(&handle);

    // let service = Proxy {
    //     client: client,
    //     director: director,
    // };

    // // println!("{}", addr);
    let http: Http = Http::new();

    let new_service = move || {
        let client = client_main.clone();

        service_fn(move |mut req| {
            let uri_string = format!("http://{}/{}",
                "127.0.0.1:4000",
                req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
            let uri = uri_string.parse().unwrap();
            *req.uri_mut() = uri;
            client.request(req)
        })
        
    };

    // let conn = http.serve_connection(socket, new_service);
    // let fut = conn.map_err(|e| eprintln!("server connection error: {}", e));

    // handle.spawn(fut);
}
