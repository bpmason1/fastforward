mod filters;

use filters::remove_hop_by_hop_headers;
use flask::httpx::{read_http_request, read_http_response};
use http::{
    header::HeaderValue,
    Request,
    Response,
    StatusCode
};
use num_cpus;
use rayon::ThreadPool;
use std::io::Write;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::string::String;
use std::process::exit;


type RequestTransform = fn(&mut Request<Vec<u8>>) -> Option<Response<Vec<u8>>>;
type ResponseTransform = fn(&mut Response<Vec<u8>>);

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

fn handle_request(stream: TcpStream, req: Request<Vec<u8>>,) {
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
            Ok(resp) => {
                write_response(resp, stream);
            },
            Err(_) => {
                let resp = Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Vec::new()).unwrap();
                write_response(resp, stream);
            }
        }
    } else {
        eprintln!("Error: sending request to client")
    }
}

fn send_request(req: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let proxy_addr = req.headers().get(http::header::HOST).unwrap();
    let proxy_stream = match TcpStream::connect(from_utf8(proxy_addr.as_bytes()).unwrap()) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("Error: could not connect to downstream service ... {}", err);
            let err_resp = Response::builder().status(StatusCode::BAD_GATEWAY).body(Vec::new()).unwrap();
            return err_resp;
        }
    };

    let err_resp = Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Vec::new()).unwrap();
    if write_request(req, proxy_stream.try_clone().unwrap()) {
        match read_http_response(proxy_stream.try_clone().unwrap()) {
            Ok(resp) => {
                // write_response(resp, stream);
                resp
            },
            Err(_) => {
                //write_response(resp, stream);
                err_resp
            }
        }
    } else {
        eprintln!("Error: sending request to client");
        err_resp
    }
}

fn handle_client(stream: TcpStream, req_trans: RequestTransform, resp_trans: ResponseTransform) {
    let mut req = read_http_request(stream.try_clone().unwrap()).unwrap();
    *req.headers_mut() = remove_hop_by_hop_headers(req.headers());

    // the short_circuit_response prevents the request from being proxied further
    let short_circuit_response = (req_trans)(&mut req);

    match short_circuit_response {
        Some(sc_resp) => {
           // Don't proxy the request ... instead use the returned response object
           if !write_response(sc_resp, stream) {
               eprintln!("Error receiving response from client");
           }
        },
        None => {
            let mut resp = send_request(req);
            (resp_trans)(&mut resp);
            write_response(resp, stream);
        }
    };
}

// ----------------------------------------------------------------------------------------
// Generic Proxy
// ----------------------------------------------------------------------------------------
pub fn generic_proxy(listen_addr: SocketAddr, req_trans: RequestTransform, resp_trans: ResponseTransform) {
    let listener: TcpListener = match TcpListener::bind(listen_addr) {
        Ok(_listener) => _listener,
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    };

    match rayon::ThreadPoolBuilder::new().num_threads(2*num_cpus::get()).build() {
        Ok(pool) => {
            pool.install( || {
                for new_stream in listener.incoming() {
                    match new_stream {
                        Ok(stream) => {
                            pool.spawn( move || {
                                handle_client(stream, req_trans, resp_trans)
                            })
                        }
                        Err(_) => {
                            eprintln!("Error accessing TcpStream in generic_proxy");
                            // TODO - decide if any further action needs to be taken here
                        }
                    }
                }
            });
        },
        Err(err) => {
            eprintln!("{}", err);
            println!("Running proxy without thread pool");
            for new_stream in listener.incoming() {
                match new_stream {
                    Ok(stream) => handle_client(stream, req_trans, resp_trans),
                    Err(_) => eprintln!("Error accessing TcpStream in generic_proxy")
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------------------
// Simple Proxy
// ----------------------------------------------------------------------------------------
pub fn simple_proxy(listen_addr: SocketAddr, proxy_addr: SocketAddr) {
    let listener: TcpListener = match TcpListener::bind(listen_addr) {
        Ok(_listener) => _listener,
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    };

    match rayon::ThreadPoolBuilder::new().num_threads(2*num_cpus::get()).build() {
        Ok(pool) => proxy_with_thread_pool(pool, listener, proxy_addr),
        Err(err) => {
            eprintln!("{}", err);
            println!("Running proxy without thread pool");
            proxy_without_thread_pool(listener, proxy_addr)
        }
    }
}

fn proxy_tcp_stream(stream: TcpStream, proxy_addr: SocketAddr, ) {
    let _proxy_add_str = format!("{}", proxy_addr);
    let proxy_addr_hdr = HeaderValue::from_str(&_proxy_add_str).unwrap();

    let mut req = read_http_request(stream.try_clone().unwrap()).unwrap();
    *req.headers_mut() = remove_hop_by_hop_headers(req.headers());

    let req_headers = req.headers_mut();
    req_headers.remove(http::header::HOST);
    req_headers.insert(http::header::HOST, proxy_addr_hdr);
    handle_request(stream, req);
}

fn proxy_without_thread_pool(listener: TcpListener, proxy_addr: SocketAddr) {
    for new_stream in listener.incoming() {
        match new_stream {
            Ok(stream) => {
                proxy_tcp_stream(stream, proxy_addr)
            }
            Err(_) => {
                eprintln!("Error accessing TcpStream in generic_proxy");
                // TODO - decide if any further action needs to be taken here
            }
        }
    }
}

fn proxy_with_thread_pool(pool: ThreadPool, listener: TcpListener, proxy_addr: SocketAddr) {
    pool.install( || {
        for new_stream in listener.incoming() {
            match new_stream {
                Ok(stream) => {
                    pool.spawn( move || {
                        proxy_tcp_stream(stream, proxy_addr)
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
