mod filters;

use filters::remove_hop_by_hop_headers;
use bottle::httpx::{read_http_request, read_http_response};
use http::{
    Request,
    Response
};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::string::String;

// use serde::{Serialize, Deserialize};
// use serde::ser::{Serialize, Serializer};


type Director = fn(&mut Request<Vec<u8>>) -> Option<Response<Vec<u8>>>;

fn write_response(resp: Response<Vec<u8>>, mut client: TcpStream) {
    let reason = match resp.status().canonical_reason() {
        Some(r) => r,
        None => ""
    };
    let resp_line = format!("{:?} {} {}\r\n", resp.version(), resp.status().as_str(), reason);
    client.write(resp_line.as_bytes());

    for (key, value) in resp.headers().iter() {
        match key.as_str() {
            "content-length" => {
                let body_size = resp.body().len();
                let h: String = format!("{}: {}\r\n", key, body_size);
                client.write(h.as_bytes());    
            },
            _ => {
                let h: String = format!("{}: {}\r\n", key, value.to_str().unwrap());
                client.write(h.as_bytes());
            }
        };
    }


    client.write(b"\r\n");
    client.write(resp.body());
} 

fn write_request(req: Request<Vec<u8>>, mut client: TcpStream) {
    let req_line = format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version());
    client.write(req_line.as_bytes());

    for (key, value) in req.headers().iter() {
        match key.as_str() {
            "content-length" => {
                let body_size = req.body().len();
                let h: String = format!("{}: {}\r\n", key, body_size);
                client.write(h.as_bytes());    
            },
            "user-agent" => {
                // ignore this header
            }
            _ => {
                let h: String = format!("{}: {}\r\n", key, value.to_str().unwrap());
                client.write(h.as_bytes());
            }
        };
    }

    client.write(b"\r\n");
    client.write(req.body());
}

fn handle_client(mut stream: TcpStream , director: Director ) {
    let mut req = read_http_request(stream.try_clone().unwrap());
    *req.headers_mut() = remove_hop_by_hop_headers(req.headers());
    match (director)(&mut req) {
        Some(resp) => {
            write_response(resp, stream);
        },
        None => {
            let proxy_addr = req.headers().get(http::header::HOST).unwrap();
            let mut proxy_stream = TcpStream::connect(from_utf8(proxy_addr.as_bytes()).unwrap()).unwrap();

            write_request(req, proxy_stream.try_clone().unwrap());

            let resp = read_http_response(proxy_stream.try_clone().unwrap());
            write_response(resp, stream);
        }
    };
}

pub fn generic_proxy(listen_addr: SocketAddr, director: Director) {
    let listener = TcpListener::bind(listen_addr).unwrap();

    // let pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();
    // pool.install( || {
        for stream in listener.incoming() {
            // pool.spawn(  || {
                handle_client(stream.unwrap(), director)
            // })
        }
    // });
}


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

