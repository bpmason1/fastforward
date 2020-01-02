extern crate bottle;
extern crate http;
extern crate rayon;

use bottle::read_http_request;
// use http::Request;
use std::io;
use std::net::{TcpListener, TcpStream};
// use std::{thread, time};
use std::str;


fn handle_client(mut stream: TcpStream) {
        // println!("Enter handle_client");

        let request = read_http_request(stream);

        // let ten_secs = time::Duration::from_millis(7000);
        // let now = time::Instant::now();
        // thread::sleep(ten_secs);

        println!("{:?}", request);
        println!("{:?}", str::from_utf8(request.body()));
}

fn main() -> io::Result<()> {

    let pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();

    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    pool.install( || {

        for stream in listener.incoming() {
            pool.spawn( || 
                handle_client(stream.unwrap())
            )
        }
    });

    Ok(())
}