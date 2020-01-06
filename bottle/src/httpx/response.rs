use super::{
    all_headers,
    array_to_vec,
    crlf,
    http,
    http_version,
    number,
    read_body,
    slash,
    token,
    space
};

use http::{Response, StatusCode, Version};
use std::io::Read;
use std::net::TcpStream;
use std::str::{self, from_utf8};


#[derive(PartialEq, Debug)]
struct ResponseLine<'a> {
    status_code: &'a str,
    version: &'a str,
    // version: HttpVersion,
}

named!( read_response_line <ResponseLine>,
    do_parse!(
        http >> slash >> version: http_version >> space >>
        status_code: number >> space >> token >> crlf >>
        (ResponseLine {status_code: status_code, version: version})
    )
);

pub fn read_http_response(mut stream: TcpStream) -> Response<Vec<u8>> {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();

    let msg = str::from_utf8(&buf).unwrap();
    let (rest1, resp_line) = read_response_line(msg.as_bytes()).unwrap();
    let (rest2, headers) = all_headers(rest1).unwrap();

    let status_code = StatusCode::from_bytes(resp_line.status_code.as_bytes()).unwrap();

    let mut response = Response::builder()
                        .status(status_code);

    match resp_line.version {
        "1.1" => {response = response.version( Version::HTTP_11 )}
        "2.0" => {response = response.version( Version::HTTP_2 )}
        _ => {}
    };

    let mut content_length = 0;
    for elem in headers.iter() {
        if elem.key.to_lowercase() == "content-length" {
            content_length = elem.value.parse::<usize>().unwrap();
        }
        response = response.header(elem.key, elem.value);
    }

    if rest2.len() < content_length {
        let mut buf2 = vec![0; content_length - rest2.len()];
        stream.read(&mut buf2).unwrap();
        let mut body_vec: Vec<u8> = array_to_vec(rest2);
        for i in buf2.iter() {
            body_vec.push(*i);
        }
        return response.body(body_vec).unwrap();
    } else {
        let (_, body) = read_body(rest2).unwrap();
        let body_vec: Vec<u8> = array_to_vec(body);
        return response.body(body_vec).unwrap();
    }
}