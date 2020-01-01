#[macro_use] extern crate nom;

// use nom::{
//     IResult,
//     branch::alt,
//     // see the "streaming/complete" paragraph lower for an explanation of these submodules
//     character::complete::char,
//     bytes::complete::{is_not, tag}
// };
use std::io::Read;
use std::net::TcpStream;
use std::str::{self, from_utf8};

named!( space, tag!(" "));

named!( read_target,
    is_not!(" ")
);

named!( read_method,
    alt!(
        tag!("DELETE") |
        tag!("GET") |
        tag!("HEAD") |
        tag!("OPTIONS") |
        tag!("PATCH") |
        tag!("POST") |
        tag!("PUT") 
    )
);

named!( read_first_line,
    do_parse!(
        method: read_method >> space >> target: read_target >> space >> (method)
    )
);


pub fn read_http_request(mut stream: TcpStream)  {
    let mut buf = [0; 512];
    stream.read(&mut buf).unwrap();

    let msg = str::from_utf8(&buf).unwrap();

    println!("{}", msg);

    // let ( rest1, method) = read_method(msg.as_bytes()).unwrap();
    // println!("{:?}", from_utf8(method));
}
