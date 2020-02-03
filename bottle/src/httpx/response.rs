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
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;
use std::{thread, time};


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

pub fn read_line_from_stream(reader: &mut BufReader<TcpStream>) -> String {
    read_line_until(reader, "\r\n")
}


pub fn read_line_until(reader: &mut BufReader<TcpStream>, terminator: &str) -> String {
    // This only works because the last character on an HTTP request line is '\n'
    let mut line = String::new();
    // /*let len =*/ reader.read_line(&mut line).unwrap();

    loop {
        let mut next_str = String::new();
        reader.read_line(&mut next_str).unwrap();
        line.push_str(next_str.as_str());
        if line.ends_with(terminator) {
            break;
        } else {
            // not all data is available in the input stream.
            // sleep 5 milliseconds before trying again
            thread::sleep(time::Duration::from_millis(5));
        }
    }

    line
}

fn read_initial_request_line(mut reader: &mut BufReader<TcpStream>) -> Builder {
    let mut line: String = String::from("");
    let resp_line = loop {
        line.push_str(read_line_from_stream(&mut reader).as_str());
        match parse_response_line(line.as_bytes()) {
            Ok((_, resp_line)) => break resp_line,
            Err(_) => {}
        }
    };

    let status_code = StatusCode::from_bytes(resp_line.status_code.as_bytes()).unwrap();

    let mut response = Response::builder().status(status_code);

    response = match resp_line.version {
        "1.1" => response.version( Version::HTTP_11 ),
        "2.0" => response.version( Version::HTTP_2 ),
        _ => response
    };

    response
}

pub fn read_http_response(stream: TcpStream) -> Result<Response<Vec<u8>>, http::Error> {
    let mut reader = BufReader::new(stream);
    let mut response = read_initial_request_line(&mut reader);

    let mut content_length = 0;

    let mut line: String = String::from("");
    loop {
        let header_line = loop {
            line.push_str(read_line_from_stream(&mut reader).as_str());
            if line.as_str() == "\r\n" {
                break None;
            }
            
            // println!("{:?}", line.as_bytes());

            if !line.ends_with("\r\n") {
                // part of the message is missing ... throttle and retry
                thread::sleep(time::Duration::from_millis(5));
                continue
            }
            
            match read_header(line.as_bytes()) {
                Ok((_, resp_line)) => break Some(resp_line),
                Err(_) => {}
            }
        };

        let done = match header_line {
            Some(elem) => {
                if elem.key.to_lowercase() == "content-length" {
                    content_length = elem.value.parse::<usize>().unwrap();
                }
                // println!("Key => {}", elem.key);
                response = response.header(elem.key, elem.value);
                line = String::from("");
                false
            },
            None => true
        };

        if done {
            break
        }
    }

    let mut body = Vec::new();
    loop {
        let mut buf2 = vec![0; content_length];
        let body_size = reader.read(&mut buf2).unwrap();
        for i in 0..body_size {
            body.push(buf2[i]);
        }

        if body.len() >= content_length {
            break;
        } else {
            // part of the message is missing ... throttle and retry
            thread::sleep(time::Duration::from_millis(5));
        }
    }
    response.body(body)
}
