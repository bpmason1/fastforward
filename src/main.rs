// use tokio::net::TcpListener;
// use std::{thread, time};
// use std::io;

// async fn process_socket<T>(socket: T) {
//     thread::spawn(move || {
//         println!("Enter hande_client");

//         let ten_secs = time::Duration::from_millis(10000);
//         let now = time::Instant::now();
//         thread::sleep(ten_secs);

//         println!("Exit hande_client");
//     });
// }

// #[tokio::main]
// async fn main() -> io::Result<()> {
//     let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

//     loop {
//         let (socket, _) = listener.accept().await?;
//         process_socket(socket).await;
//     }
// }

extern crate http;
// #[macro_use] extern crate http_handler;
#[macro_use] extern crate nom;
extern crate rayon;

use http::Request;
use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};
use std::{thread, time};
use std::str;
use nom::{
    IResult,
    branch::alt,
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
    bytes::complete::{is_not, tag}
};
// use http_handler::grammar::http_message;

fn request_target(input: &str) -> IResult<&str, &str> {
    is_not(" ")(input)
}

fn read_method(input: &str) -> IResult<&str, &str> {
    let method_tuple = (
        tag("DELETE"),
        tag("GET"),
        tag("HEAD"),
        tag("OPTIONS"),
        tag("PATCH"),
        tag("POST"),
        tag("PUT"),
    );

    alt(method_tuple)(input)
}

fn read_http_version(input: &str) -> IResult<&str, &str> {
    tag("HTTP")(input)
}

fn space(input: &str) -> IResult<&str, &str> {
    tag(" ")(input)
}

// fn http_version((input: &str) -> IResult<&str, &str> {

// }

// named!(pub space, map_res!(tag!(" "), str::from_utf8));

named!(pub http_name, tag!("HTTP"));

// named!(pub request_target <&str>, map_res!(is_not!(" "), str::from_utf8));

fn tcp_stream_to_http_request(mut stream: TcpStream) -> Request<()> {
    let mut request = Request::builder();

    let mut buf = [0; 512];
    stream.read(&mut buf).unwrap();

    let msg = str::from_utf8(&buf).unwrap();

    // let (_, foo) = http_message(msg.as_bytes()).unwrap();
    let ( rest1, method) = read_method(msg).unwrap();
    let (rest2, _) = space(rest1).unwrap();
    let (rest3, target) = request_target(rest2).unwrap();
    let (rest4, _) = space(rest3).unwrap();
    read_http_version(rest4);

    // println!("{:?}", str::from_utf8(rest3));
    println!("{:?}", rest4);

    request.method(method).body(()).unwrap()
}

fn handle_client(mut stream: TcpStream) {
        println!("Enter hande_client");

        let request = tcp_stream_to_http_request(stream);
        // let mut buf = Vec::new();
        // stream.read_to_end(&mut buf); // match


        // let ten_secs = time::Duration::from_millis(7000);
        // let now = time::Instant::now();
        // thread::sleep(ten_secs);

        // println!("{:?}", request);
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