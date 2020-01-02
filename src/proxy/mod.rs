mod filters;

use filters::remove_hop_by_hop_headers;
use bottle::read_http_request;
use http::Request;
use std::fmt;
use std::io::Write;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::string::String;
// use serde::{Serialize, Deserialize};
// use serde::ser::{Serialize, Serializer};

type Director = fn(&mut Request<Vec<u8>>); // -> Option<ResponseFuture>;

fn serialize_request(req: Request<Vec<u8>>) -> Vec<String> {
    // serializes everything but the request body (for performance reasons)
    let mut vector = Vec::new();
    let req_line = format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version());
    vector.push(req_line);

    for (key, value) in req.headers().iter() {
        if key.as_str().to_lowercase() != "content-length" {
            let h: String = format!("{}: {}\r\n", key, value.to_str().unwrap());
            vector.push(h);
        } else {
            let body_size = req.body().len();
            let h: String = format!("{}: {}\r\n", key, body_size);
            vector.push(h);
        }
    }

    vector.push(format!("\r\n"));
    vector.push(format!("{:?}", req.body()));
    vector
}

// impl Serialize for Request<Vec<u8>> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//     }
// }


fn handle_client(mut stream: TcpStream , director: Director ) {
    let mut req = read_http_request(stream.try_clone().unwrap());
    *req.headers_mut() = remove_hop_by_hop_headers(req.headers());
    (director)(&mut req);

    // let first_line = format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version());
    // stream.write(first_line.as_bytes()).unwrap();

    for s in serialize_request(req) {
        println!("{}", s);
        stream.write(s.as_bytes()).unwrap();
    }
    // stream.write("\r\n".as_bytes()).unwrap();
}

pub fn generic_proxy(/*listen_addr: SocketAddr,*/ director: Director) {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();
    // pool.install( || {
        for stream in listener.incoming() {
            // pool.spawn(  || {
                handle_client(stream.unwrap(), director)
            // })
        }
    // });
}

// pub fn generic_proxy(listen_addr: SocketAddr, director: Director) {
//     let client_main = Client::new();

//     let new_service = move || {
//         let client = client_main.clone();

//         service_fn(move |mut req| {
//             *req.headers_mut() = remove_hop_by_hop_headers(req.headers());
//             (director)(&mut req);
//             // let resp = (director)(&mut req);
//             // match resp {
//             //     Some(r) => { r }
//             //     None => { client.request(req) }
//             // }
//             client.request(req)
//         })
        
//     };

//     let server = Server::bind(&listen_addr)
//         .serve(new_service)
//         .map_err(|e| eprintln!("server error: {}", e));


//     rt::run(server);
// }

// pub fn simple_proxy(listen_addr: SocketAddr, proxy_addr: SocketAddr) {
//     let client_main = Client::new();
    
//     let new_service = move || {
//         let client = client_main.clone();

//         service_fn(move |mut req| {
//             *req.headers_mut() = remove_hop_by_hop_headers(req.headers());
//             let scheme = req.uri().scheme_str().unwrap();
//             let uri_string = format!(
//                 "{}://{}/{}",
//                 scheme,
//                 proxy_addr,
//                 req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
//             let uri = uri_string.parse().unwrap();
//             *req.uri_mut() = uri;

//             client.request(req)
//         })
//     };

//     let server = Server::bind(&listen_addr)
//         .serve(new_service)
//         .map_err(|e| eprintln!("server error: {}", e));

//     rt::run(server);
// }

