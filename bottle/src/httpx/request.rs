use super::{
    all_headers,
    array_to_vec,
    crlf,
    http,
    http_version,
    number,
    read_body,
    read_method,
    slash,
    space,
    spaces,
    to_space,
    token
};

use http::{Request, StatusCode, Version};
use std::io::Read;
use std::net::TcpStream;
use std::str::{self, from_utf8};

#[derive(PartialEq, Debug)]
struct RequestLine<'a> {
    method: &'a str,
    target: &'a str, // [u8],
    version: &'a str,
    // version: HttpVersion,
}

named!( read_request_line <RequestLine>,
    do_parse!(
        method: read_method >> opt!(spaces) >> target: to_space >> opt!(spaces) >>
        http >> slash >> version: http_version >> crlf >>
        (RequestLine {method: method, target: target , version: version})
    )
);

pub fn read_http_request(mut stream: TcpStream) -> Request<Vec<u8>> {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();

    let msg = str::from_utf8(&buf).unwrap();
    let (rest1, req_line) = read_request_line(msg.as_bytes()).unwrap();
    let (rest2, headers) = all_headers(rest1).unwrap();
    // println!("{:?}", from_utf8(rest3));
    // println!("{:?}", from_utf8(body));
    // println!("{:?}", headers);

    let mut request = Request::builder()
                        .method(req_line.method)
                        .uri(req_line.target);
    match req_line.version {
        "1.1" => {request = request.version( Version::HTTP_11 )}
        "2.0" => {request = request.version( Version::HTTP_2 )}
        _ => {}
    };

    let mut content_length = 0;
    for elem in headers.iter() {
        if elem.key.to_lowercase() == "content-length" {
            content_length = elem.value.parse::<usize>().unwrap();
        }
        request = request.header(elem.key, elem.value);
    }

    if rest2.len() < content_length {
        let mut buf2 = vec![0; content_length - rest2.len()];
        stream.read(&mut buf2).unwrap();
        let mut body_vec: Vec<u8> = array_to_vec(rest2);
        for i in buf2.iter() {
            body_vec.push(*i);
        }
        return request.body(body_vec).unwrap();
    } else {
        let (_, body) = read_body(rest2).unwrap();
        let body_vec: Vec<u8> = array_to_vec(body);
        return request.body(body_vec).unwrap();
    }

}