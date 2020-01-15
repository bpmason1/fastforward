use super::{
    all_headers,
    array_to_vec,
    http,
    http_version,
    read_body,
    read_header,
    token
};

use crate::combinators::{
    crlf,
    number,
    slash,
    spaces
};

use http::{Response, StatusCode, Version};
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;
use std::str::{self, from_utf8};
use std::{thread, time};


#[derive(PartialEq, Debug)]
struct ResponseLine<'a> {
    status_code: &'a str,
    version: &'a str,
    // version: HttpVersion,
}

named!( read_response_line <ResponseLine>,
    do_parse!(
        http >> slash >> version: http_version >> opt!(spaces) >>
        status_code: number >> opt!(spaces) >> token >> crlf >>
        (ResponseLine {status_code: status_code, version: version})
    )
);

pub fn read_line_from_stream(reader: &mut BufReader<TcpStream>) -> String {
    // This only works because the last character on an HTTP request line is '\n'
    let mut line = String::new();
    let len = reader.read_line(&mut line).unwrap();
    println!("{}", line);
    line
}

pub fn read_http_response(mut stream: TcpStream) -> Response<Vec<u8>> {
    // hack because I'm not properly handling slow streams
    let half_sec = time::Duration::from_millis(500);
    thread::sleep(half_sec);

    let mut reader = BufReader::new(stream);

    let mut line = read_line_from_stream(&mut reader);
    let (_, resp_line) = read_response_line(line.as_bytes()).unwrap();

    let status_code = StatusCode::from_bytes(resp_line.status_code.as_bytes()).unwrap();

    let mut response = Response::builder()
                        .status(status_code);

    match resp_line.version {
        "1.1" => {response = response.version( Version::HTTP_11 )}
        "2.0" => {response = response.version( Version::HTTP_2 )}
        _ => {}
    };

    let mut rest1 = [0; 1024];
    reader.read(&mut rest1).unwrap();
    let (rest2, headers) = all_headers(&rest1).unwrap();

    let mut content_length = 0;
    for elem in headers.iter() {
        if elem.key.to_lowercase() == "content-length" {
            content_length = elem.value.parse::<usize>().unwrap();
        }
        response = response.header(elem.key, elem.value);
    }

    if rest2.len() < content_length {
        let mut buf2 = vec![0; content_length - rest2.len()];
        reader.read(&mut buf2).unwrap();
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