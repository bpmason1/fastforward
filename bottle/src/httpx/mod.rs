mod request;
mod response;

pub use request::read_http_request;
pub use response::read_http_response;

use std::str::{self, from_utf8};
use nom::character::is_alphanumeric;
use std::vec::Vec;

#[derive(PartialEq, Debug)]
struct Header<'b> {
    key: &'b str,
    value: &'b str,
}

fn is_token_char(i: u8) -> bool {
    is_alphanumeric(i) ||
    b"!#$%&'*+-.^_`|~".contains(&i)
  }


fn is_digit_char(i: u8) -> bool {
    b"0123456789".contains(&i)
  }

named!(token <&str>,
    map_res!(
        take_while!(is_token_char),
        from_utf8
    )
);

named!(number <&str>,
    map_res!(
        take_while!(is_digit_char),
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

fn array_to_vec(arr: &[u8]) -> Vec<u8> {
    let mut vector = Vec::new();
    for i in arr.iter() {
        vector.push(*i);
    }
    vector
}
