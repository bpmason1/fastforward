use nom::character::{is_alphanumeric, is_space};
use std::str::{self, from_utf8};

// ***************************************************************************
// scalar combinators
// ***************************************************************************
named!(pub crlf, tag!("\r\n"));

named!(pub colon, tag!(":"));

named!(pub slash, tag!("/"));

named!(pub space, tag!(" "));

// ***************************************************************************
// repeated combinators
// ***************************************************************************
named!( pub spaces, take_while!(is_space));

// ***************************************************************************
// classifiers
// ***************************************************************************
pub fn is_digit_char(i: u8) -> bool {
    b"0123456789".contains(&i)
 }

 named!(pub number <&str>,
    map_res!(
        take_while!(is_digit_char),
        from_utf8
    )
);

// ***************************************************************************
// http related combinators
// ***************************************************************************
named!( pub http, tag!("HTTP"));

// allows ISO-8859-1 characters in header values
// this is allowed in RFC 2616 but not in rfc7230
#[cfg(  feature = "tolerant-http1-parser")]
pub fn is_header_value_char(i: u8) -> bool {
  i == 9 || (i >= 32 && i <= 126) || i >= 160
}

#[cfg(not(feature = "tolerant-http1-parser"))]
pub fn is_header_value_char(i: u8) -> bool {
  i == 9 || (i >= 32 && i <= 126)
}

named!( pub header_value <&str>,
    map_res!(
        take_while!(is_header_value_char),
        from_utf8
    )
);

named!( pub http_method <&str>,
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

fn is_body_value_char(i: u8) -> bool {
    i > 0
}

named!( pub read_body,
    take_while!(is_body_value_char)
);

fn is_http_header_name_char(i: u8) -> bool {
    is_alphanumeric(i) ||
    b"!#$%&'*+-.^_`|~".contains(&i)
  }

  named!(pub http_header_name <&str>,
    map_res!(
        take_while!(is_http_header_name_char),
        from_utf8
    )
);