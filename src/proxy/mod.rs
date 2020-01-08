mod filters;

use filters::remove_hop_by_hop_headers;
use bottle::httpx::{read_http_request, read_http_response};
use http::{
    Request,
    Response
};
use num_cpus;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::string::String;


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

    let pool = rayon::ThreadPoolBuilder::new().num_threads(2*num_cpus::get()).build().unwrap();
    pool.install( || {
        for stream in listener.incoming() {
            pool.spawn( move || {
                handle_client(stream.unwrap(), director)
            })
        }
    });
}

