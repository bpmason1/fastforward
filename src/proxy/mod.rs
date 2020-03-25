mod filters;

use filters::remove_hop_by_hop_headers;
use flask::httpx::{read_http_request, read_http_response};
use http::{
    Request,
    Response,
    StatusCode
};
use num_cpus;
use std::io::Write;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::string::String;


type Director = fn(&mut Request<Vec<u8>>) -> Option<Response<Vec<u8>>>;

fn write_response(resp: Response<Vec<u8>>, mut client: TcpStream) -> bool {
    let reason = match resp.status().canonical_reason() {
        Some(r) => r,
        None => ""
    };
    let resp_line = format!("{:?} {} {}\r\n", resp.version(), resp.status().as_str(), reason);
    if client.write(resp_line.as_bytes()).is_err() {
        return false;
    }

    for (key, value) in resp.headers().iter() {
        match key.as_str() {
            "content-length" => {
                let body_size = resp.body().len();
                let h: String = format!("{}: {}\r\n", key, body_size);
                if client.write(h.as_bytes()).is_err() {
                    return false;
                }
            },
            _ => {
                let h: String = format!("{}: {}\r\n", key, value.to_str().unwrap());
                if client.write(h.as_bytes()).is_err() {
                    return false;
                }
            }
        };
    }

    if client.write(b"\r\n").is_err() {
        return false;
    }

    client.write(resp.body()).is_ok()
} 

fn write_request(req: Request<Vec<u8>>, mut client: TcpStream) -> bool {
    let req_line = format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version());
    if client.write(req_line.as_bytes()).is_err() {
        return false;
    }

    for (key, value) in req.headers().iter() {
        match key.as_str() {
            "content-length" => {
                let body_size = req.body().len();
                let h: String = format!("{}: {}\r\n", key, body_size);
                if client.write(h.as_bytes()).is_err() {
                    return false;
                }
            },
            "user-agent" => {
                // ignore this header
            }
            _ => {
                let h: String = format!("{}: {}\r\n", key, value.to_str().unwrap());
                if client.write(h.as_bytes()).is_err() {
                    return false;
                }
            }
        };
    }

    if client.write(b"\r\n").is_err() {
        return false;
    }

    client.write(req.body()).is_ok()
}

fn handle_client(stream: TcpStream , director: Director ) {
    let mut req = read_http_request(stream.try_clone().unwrap()).unwrap();
    *req.headers_mut() = remove_hop_by_hop_headers(req.headers());
    match (director)(&mut req) {
        Some(resp) => {
            if !write_response(resp, stream) {
                eprintln!("Error receiving response from client");
            }
        },
        None => {
            let proxy_addr = req.headers().get(http::header::HOST).unwrap();
            let proxy_stream = match TcpStream::connect(from_utf8(proxy_addr.as_bytes()).unwrap()) {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error: could not connect to downstream service ... {}", err);
                    return;
                }
            };

            if write_request(req, proxy_stream.try_clone().unwrap()) {
                match read_http_response(proxy_stream.try_clone().unwrap()) {
                    Ok(resp) => { write_response(resp, stream); },
                    Err(_) => {
                        let resp = Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Vec::new()).unwrap();
                        write_response(resp, stream);
                    }
                }
            } else {
                eprintln!("Error: sending request to client")
            }
        }
    };
}

pub fn generic_proxy(listen_addr: SocketAddr, director: Director) {
    let listener = TcpListener::bind(listen_addr).unwrap();

    let pool = rayon::ThreadPoolBuilder::new().num_threads(2*num_cpus::get()).build().unwrap();
    pool.install( || {
        for new_stream in listener.incoming() {
            match new_stream {
                Ok(stream) => {
                    pool.spawn( move || {
                        handle_client(stream, director)
                    })
                }
                Err(_) => {
                    eprintln!("Error accessing TcpStream in generic_proxy");
                    // TODO - decide if any further action needs to be taken here
                }
            }
        }
    });
}

