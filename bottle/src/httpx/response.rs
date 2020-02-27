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
    let mut line: String = String::from("");
    reader.read_line(&mut line).unwrap();

    let (_, resp_line) = parse_response_line(line.as_bytes()).unwrap();

    let status_code_bytes = resp_line.status_code.as_bytes();
    let status_code = StatusCode::from_bytes(status_code_bytes)?;

    let mut response = Response::builder().status(status_code);

    response = match resp_line.version {
        "0.9" => response.version( Version::HTTP_09 ),
        "1.0" => response.version( Version::HTTP_10 ),
        "1.1" => response.version( Version::HTTP_11 ),
        "2.0" => response.version( Version::HTTP_2 ),
        "3.0" => response.version( Version::HTTP_3 ),
        _ => { response }  // I don't know the http version so skip it
    };

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

        let (_, header_line) = read_header(line.as_bytes()).unwrap();
        // println!("{:?}", line.as_bytes());


        if header_line.key.to_lowercase() == "content-length" {
            content_length = header_line.value.parse::<usize>().unwrap();
        }
        // println!("Key => {}", elem.key);
        response = response.header(header_line.key, header_line.value);
    }

    let mut body = Vec::new();
    let mut buf2 = vec![0; content_length];
    let body_size = reader.read(&mut buf2).unwrap();
    for i in 0..body_size {
        body.push(buf2[i]);
    }

    // if body.len() >= content_length {
    //     break;
    // } else {
    //     // part of the message is missing ... throttle and retry
    //     thread::sleep(time::Duration::from_millis(5));
    // }
    response.body(body)
}

pub fn read_http_response(stream: TcpStream) -> Result<Response<Vec<u8>>, http::Error> {
    let mut reader: BufReader<TcpStream> = BufReader::new(stream);

    _read_http_response(&mut reader)
}
