mod filters;

use filters::remove_hop_by_hop_headers;
use bottle::read_http_request;
use http::{
    header::HeaderName,
    Request,
    Response
};
use std::fmt;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::string::String;

// use serde::{Serialize, Deserialize};
// use serde::ser::{Serialize, Serializer};

lazy_static! {
    pub static ref FF_PROXT_HOST: HeaderName = {
        HeaderName::from_lowercase(b"ff-proxy-host").unwrap()
    };
}

type Director = fn(&mut Request<Vec<u8>>) -> Option<Response<Vec<u8>>>;

fn serialize_request(req: Request<Vec<u8>>) -> Vec<String> {
    // serializes everything but the request body (for performance reasons)
    let mut vector = Vec::new();
    let req_line = format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version());
    vector.push(req_line);

    for (key, value) in req.headers().iter() {
        match key.as_str() {
            "content-length" => {
                let body_size = req.body().len();
                let h: String = format!("{}: {}\r\n", key, body_size);
                vector.push(h);    
            },
            "host" => {
                let h: String = format!("{}: {}\r\n", key, "127.0.0.1:4000");
                vector.push(h);
            },
            "user-agent" => {
                // ignore this header
            }
            _ => {
                let h: String = format!("{}: {}\r\n", key, value.to_str().unwrap());
                vector.push(h);
            }
        };
    }

    vector.push(format!("\r\n"));
    vector.push(format!("{:?}", req.body()));  // TODO - don't send debug representation of byte array (this is wrong!!)
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
    match (director)(&mut req) {
        Some(resp) => {
            // TODO - serialize the response and write it to the open TcpStream
        },
        None => {
            let proxy_addr = req.headers().get(FF_PROXT_HOST.clone()).unwrap();
            let mut client = TcpStream::connect(from_utf8(proxy_addr.as_bytes()).unwrap()).unwrap();

            for s in serialize_request(req) {
                println!("{}", s);
                client.write(s.as_bytes()).unwrap();
            }

            // TODO - reconstruct the HTTP response to ensure the entire message is returned
            const BUF_SIZE: usize = 1024;
            loop {
                let mut buf = [0; BUF_SIZE];
                let len = client.read(&mut buf).expect("read failed");
    
                if len > 0 {
                    stream.write(&buf).unwrap();
                }

                if len < BUF_SIZE {
                    break;
                }
            }
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

