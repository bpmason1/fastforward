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
use std::ptr;
use std::cmp::min;


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

        let (_, header_line) = read_header(line.as_bytes()).unwrap();
        // println!("{:?}", line.as_bytes());


        if header_line.key.to_lowercase() == "content-length" {
            content_length = header_line.value.parse::<usize>().unwrap();
        }
        // println!("Key => {}", elem.key);
        response = response.header(header_line.key, header_line.value);
    }

    let mut bytes_read = 0;
    let mut body = vec![0; content_length];
    let mut buf2 = vec![0; content_length];
    while bytes_read < content_length {
        let body_size = reader.read(&mut buf2).unwrap();
        let bytes_to_copy = min(body_size, content_length - bytes_read);
        unsafe {
            ptr::copy_nonoverlapping(buf2.as_ptr(), body.as_mut_ptr().offset(bytes_read as isize), bytes_to_copy);
        }
        bytes_read += body_size;
        // std::thread::sleep(std::time::Duration::from_millis(5));
    }

    response.body(body)
}

pub fn read_http_response(stream: TcpStream) -> Result<Response<Vec<u8>>, http::Error> {
    let mut reader: BufReader<TcpStream> = BufReader::new(stream);

    _read_http_response(&mut reader)
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use http::{StatusCode};
    use mockito::{mock, server_address};
    use std::net::TcpStream;
    use rand::{Rng, thread_rng};
    use rand::distributions::Alphanumeric;
    
    #[test]
    fn test_minimal_get_request() {
        let _mock = mock("GET", "/hello").create();
        let mut stream = TcpStream::connect(server_address()).unwrap();
        stream.write_all("GET /hello HTTP/1.1\r\n\r\n".as_bytes()).unwrap();
        let resp = read_http_response(stream).unwrap();
        let content_length = resp.headers()[http::header::CONTENT_LENGTH].to_str().unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.version(), Version::HTTP_11);
        assert_eq!(content_length, "0");
        assert!(resp.body().is_empty());

        _mock.assert();
    }

    #[test]
    fn test_post_request_with_body() {
        let mut rng = thread_rng();
        let rand_len = rng.gen_range(10, 20);
        let rand_body: String = rng
            .sample_iter(Alphanumeric)
            .take(rand_len.clone())
            .collect();

        let _mock = mock("POST", "/foo/bar").with_body(rand_body.clone()).create();

        // Place a request
        let mut stream = TcpStream::connect(server_address()).unwrap();
        stream.write_all("POST /foo/bar HTTP/1.1\r\n\r\n".as_bytes()).unwrap();
        stream.flush().unwrap();

        // Read the response
        let resp = read_http_response(stream).unwrap();
        let body: String = String::from_utf8(resp.body().to_vec()).unwrap();
        let content_length = resp.headers()[http::header::CONTENT_LENGTH].to_str().unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.version(), Version::HTTP_11);
        assert_eq!(content_length, rand_len.to_string());
        assert_eq!(body.clone(), rand_body);
        assert_eq!(body.len(), rand_len);

        _mock.assert();
    }

    #[test]
    fn test_post_response_with_large_body() {
        let mut rng = thread_rng();
        let rand_len: usize = rng.gen_range(1e5 as usize, 1e6 as usize);
        let rand_body: String = rng
            .sample_iter(Alphanumeric)
            .take(rand_len.clone())
            .collect();

        let _mock = mock("POST", "/foo-bar").with_body(rand_body.clone()).create();  // .expect_at_most(1).create();

        // Place a request
        let mut stream = TcpStream::connect(server_address()).unwrap();
        stream.write_all("POST /foo-bar HTTP/1.1\r\n\r\n".as_bytes()).unwrap();
        stream.flush().unwrap();

        // Read the response
        let resp_result = read_http_response(stream);
        assert_eq!(resp_result.is_ok(), true);
        let resp = resp_result.unwrap();

        let body: String = String::from_utf8(resp.body().to_vec()).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body.len(), rand_len);
        assert_eq!(body, rand_body);

        _mock.assert();
    }
}
