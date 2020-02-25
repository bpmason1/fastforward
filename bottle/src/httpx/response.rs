use super::{
    http,
    http_version,
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
use http::response::Builder;
// use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;
use std::{thread, time};
use streambuf::{
    StreamBuf,
    self,
};


#[derive(PartialEq, Debug)]
struct ResponseLine<'a> {
    status_code: &'a str,
    version: &'a str,
}

named!( parse_response_line <ResponseLine>,
    do_parse!(
        http >> slash >> version: http_version >> opt!(spaces) >>
        status_code: number >> opt!(spaces) >> token >> crlf >>
        (ResponseLine {status_code: status_code, version: version})
    )
);

fn read_initial_request_line(mut reader: &mut StreamBuf) -> Builder {
    let line: Vec<u8> = reader.read_line().unwrap();
    let (_, resp_line) = parse_response_line(line.as_slice()).unwrap();
  
    // let mut line: String = String::from("");
    // let resp_line = loop {
    //     line.push_str(read_line(&mut reader).unwrap().as_str());
    //     match parse_response_line(line.as_bytes()) {
    //         Ok((_, resp_line)) => break resp_line,
    //         Err(_) => {}
    //     }
    // };

    let status_code = StatusCode::from_bytes(resp_line.status_code.as_bytes()).unwrap();

    let mut response = Response::builder().status(status_code);

    response = match resp_line.version {
        "1.1" => response.version( Version::HTTP_11 ),
        "2.0" => response.version( Version::HTTP_2 ),
        _ => response
    };

    response
}

pub fn read_http_response(mut stream: TcpStream) -> Result<Response<Vec<u8>>, http::Error> {
    let mut reader = streambuf::new(stream.try_clone().unwrap());

    // let mut buffer = [0; 50];
    // stream.read(&mut buffer);
    // println!("{}", std::str::from_utf8(&buffer).unwrap());

    let mut response = read_initial_request_line(&mut reader);

    let mut content_length = 0;

    loop {
        let resp_line = reader.read_until(b"\r\n").unwrap();
        let resp_bytes = resp_line.as_slice();
        if resp_bytes == b"\r\n" {
            break
        }

        let (_, elem) = read_header(resp_bytes).unwrap();
        if elem.key.to_lowercase() == "content-length" {
            content_length = elem.value.parse::<usize>().unwrap();
        }
        // println!("Key => {}", elem.key);
        response = response.header(elem.key, elem.value);
    }

    let mut body = Vec::new();

    let body_vec = reader.read_num_bytes(content_length).unwrap();

    for i in 0..content_length {
        body.push(body_vec[i]);
    }

    response.body(body)
}
