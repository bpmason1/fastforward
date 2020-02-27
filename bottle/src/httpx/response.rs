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
use std::cmp::max;
use std::io::{
    self,
    BufReader,
    prelude::*
};
use std::net::TcpStream;

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

fn read_initial_request_line(reader: &mut BufReader<TcpStream>) -> Result<Builder, http::Error> {
    let mut response = Response::builder();

    let mut line: String = String::from("");
    match reader.read_line(&mut line) {
        Ok(_) => {
            let (_, resp_line) = parse_response_line(line.as_bytes()).unwrap();

            let status_code_bytes = resp_line.status_code.as_bytes();
            let status_code = StatusCode::from_bytes(status_code_bytes)?;

            response = response.status(status_code);

            response = match resp_line.version {
                "0.9" => response.version( Version::HTTP_09 ),
                "1.0" => response.version( Version::HTTP_10 ),
                "1.1" => response.version( Version::HTTP_11 ),
                "2.0" => response.version( Version::HTTP_2 ),
                "3.0" => response.version( Version::HTTP_3 ),
                _ => { response }  // I don't know the http version so skip it
            };
        },
        Err(_) => {}
    }
    Ok(response)
}

fn _read_http_response(reader: &mut BufReader<TcpStream>) -> Result<Response<Vec<u8>>, http::Error> {
    let mut response = read_initial_request_line(reader)?;

    let mut content_length = 0;

    loop {
        let mut line: String = String::from("");
        let num_bytes_result: Result<usize, io::Error> = reader.read_line(&mut line);

        let num_bytes = num_bytes_result.unwrap();

        if num_bytes == 2 && line.as_str() == "\r\n" {
            break;
        }

        match read_header(line.as_bytes()) {
            Ok((_, header_line)) => {
                if header_line.key.to_lowercase() == "content-length" {
                    match header_line.value.parse::<usize>() {
                        Ok(value) => {
                            content_length = max(content_length, value);
                        },
                        Err(_) => { /* do nothing */ }
                    }
                } else {
                    response = response.header(header_line.key, header_line.value);
                }
                // println!("Key => {}", elem.key);
            },
            Err(_) => {
                // TODO - don't ignore garbled/malformed response headers
                content_length = 0;
                response = response.status(StatusCode::INTERNAL_SERVER_ERROR);
                break;
            }
        }
    }  // end-loop

    let mut body = Vec::new();
    if content_length > 0 {
        let mut buf2 = vec![0; content_length];
        match reader.read(&mut buf2) {
            Ok(body_size) => {
                if body_size == content_length {
                    // TODO - find a more efficient way to do this
                    for i in 0..body_size {
                        body.push(buf2[i]);
                    }
                } else {
                    content_length = 0;
                    response = response.status(StatusCode::REQUEST_TIMEOUT);
                }
            },
            Err(_) => {
                content_length = 0;
            }
        }  // end-match
    }  // end-if

    // this ensures that every non-error response has exactly 1 content-length header
    response = response.header("content-length", content_length);

    response.body(body)
}

pub fn read_http_response(stream: TcpStream) -> Result<Response<Vec<u8>>, http::Error> {
    let mut reader: BufReader<TcpStream> = BufReader::new(stream);

    _read_http_response(&mut reader)
}
