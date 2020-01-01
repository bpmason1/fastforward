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
use nom::character::is_alphanumeric;

#[derive(PartialEq, Debug)]
pub struct Header<'b> {
    pub key: &'b str,
    pub value: &'b str,
}

#[derive(PartialEq, Debug)]
pub struct RequestLine<'a> {
    pub method: &'a str,
    pub target: &'a str, // [u8],
    pub version: &'a str,
    // pub version: HttpVersion,
}

fn is_token_char(i: u8) -> bool {
    is_alphanumeric(i) ||
    b"!#$%&'*+-.^_`|~".contains(&i)
  }

named!(token <&str>,
    map_res!(
        take_while!(is_token_char),
        from_utf8
    )
);

// allows ISO-8859-1 characters in header values
// this is allowed in RFC 2616 but not in rfc7230
// cf https://github.com/sozu-proxy/sozu/issues/479
#[cfg(feature = "tolerant-http1-parser")]
fn is_header_value_char(i: u8) -> bool {
  i == 9 || (i >= 32 && i <= 126) || i >= 160
}

#[cfg(not(feature = "tolerant-http1-parser"))]
fn is_header_value_char(i: u8) -> bool {
  i == 9 || (i >= 32 && i <= 126)
}

fn is_body_value_char(i: u8) -> bool {
    i > 0
}

named!(pub crlf, tag!("\r\n"));

named!( colon, tag!(":"));

named!( http, tag!("HTTP"));

named!( slash, tag!("/"));

named!( space, tag!(" "));

named!( http_version <&str>,
    map_res!(
        take!(3),
        from_utf8
    )
);

named!( header_value <&str>,
    map_res!(
        take_while!(is_header_value_char),
        from_utf8
    )
);

named!( to_colon <&str>,
    map_res!(
        is_not!(":"),
        from_utf8
    )
);

named!( to_space <&str>,
    map_res!(
        is_not!(" "),
        from_utf8
    )
);

named!( read_body,
    take_while!(is_body_value_char)
);

named!( read_method <&str>,
    map_res!(
        alt!(
            tag!("CONNECT") |
            tag!("DELETE") |
            tag!("GET") |
            tag!("HEAD") |
            tag!("OPTIONS") |
            tag!("PATCH") |
            tag!("POST") |
            tag!("PUT") |
            tag!("TRACE") 
        ),
        from_utf8
    )
);

named!( read_header <Header>,
    do_parse!(
        key: token >> colon >> space >> value: header_value >> crlf >>
        (Header {key: key, value: value})
    )
);

named!(all_headers< Vec<Header> >,
    terminated!(
        many0!(read_header),
        opt!(crlf)
    )
);

named!( read_first_line <RequestLine>,
    do_parse!(
        method: read_method >> space >> target: to_space >> space >>
        http >> slash >> version: http_version >> crlf >>
        (RequestLine {method: method, target: target , version: version})
    )
);


pub fn read_http_request(mut stream: TcpStream)  {
    let mut buf = [0; 512];
    stream.read(&mut buf).unwrap();

    let msg = str::from_utf8(&buf).unwrap();
    let (rest1, req_line) = read_first_line(msg.as_bytes()).unwrap();
    let (rest2, headers) = all_headers(rest1).unwrap();
    let (rest3, body) = read_body(rest2).unwrap();
    println!("{:?}", from_utf8(rest3));
    println!("{:?}", from_utf8(body));

    // let ( rest1, method) = read_method(msg.as_bytes()).unwrap();
    // println!("{:?}", from_utf8(method));
}
