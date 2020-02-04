extern crate twoway;

use std::io::BufReader;
use std::io::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::{self, Duration};

const CRLF: &str = "\r\n";

pub struct StreamBuf {
    inner: TcpStream,
    buffer: Vec<u8>,
}

pub fn new(inner: TcpStream) -> StreamBuf {
    StreamBuf{
        inner: inner,
        buffer: Vec::new(),
    }
}

impl StreamBuf {
    fn read_inner_to_buffer(&mut self) -> Result<usize, Error> {
        let mut tmp_vec = Vec::new();
        let result = self.inner.read( &mut tmp_vec );
        if result.is_ok() {
            self.buffer.append(&mut tmp_vec);
        }
        result
    }

    pub fn read_line(&mut self) -> Option<Vec<u8>> {
        self.read_until(b"\r\n")
    }

    // TODO - add timeout
    pub fn read_until(&mut self, terminator: &[u8]) -> Option<Vec<u8>> {
        let mut result = None;
        while result.is_none() {
            result = self.read_once_until(terminator);
        }
        result
    }

    fn read_once_until(&mut self, terminator: &[u8]) -> Option<Vec<u8>> {
        self.read_inner_to_buffer();
        read_once_until(&mut self.buffer, terminator)
    }

}

fn read_once_until(haystack: &mut Vec<u8>, needle: &[u8]) -> Option<Vec<u8>> {
    if haystack.len() < needle.len() {
        return None;
    }

    let start_idx_opt = twoway::find_bytes(haystack.as_slice(), needle);
    let result: Option<Vec<u8>> = match start_idx_opt {
        Some(start_idx) => {
            let end_idx = start_idx + needle.len();
            let data: Vec<u8> = haystack.drain(..end_idx).collect();
            Some(data)
        },
        None => None
    };
    result
}

#[test]
fn test_read_once_until() {
    unsafe {
        let mut text = String::from("Hello World");
        let mut haystack: Vec<u8> = text.as_mut_vec().to_vec();
        assert_eq!(read_once_until(&mut haystack.to_vec(), b"not there"), None);
    }

    unsafe {
        let mut text = String::from("Hello World");
        let mut haystack: Vec<u8> = text.as_mut_vec().to_vec();
        let resp = read_once_until(&mut haystack, b"Wo");
        assert_eq!(resp.unwrap().as_slice(), b"Hello Wo");
        assert_eq!(haystack.as_slice(), b"rld");
    }

    unsafe {
        let mut text = String::from("Hello World");
        let mut haystack: Vec<u8> = text.as_mut_vec().to_vec();
        let resp = read_once_until(&mut haystack, b"Hello");
        assert_eq!(resp.unwrap().as_slice(), b"Hello");
        assert_eq!(haystack.as_slice(), b" World");
    }

    unsafe {
        let mut text = String::from("Foo Bar Bar Foo");
        let mut haystack: Vec<u8> = text.as_mut_vec().to_vec();
        let resp = read_once_until(&mut haystack, b"Bar");
        assert_eq!(resp.unwrap().as_slice(), b"Foo Bar");
        assert_eq!(haystack.as_slice(), b" Bar Foo");
    }

    unsafe {
        let mut text = String::from("XYYZ");
        let mut haystack: Vec<u8> = text.as_mut_vec().to_vec();
        let resp = read_once_until(&mut haystack, b"Y");
        assert_eq!(resp.unwrap().as_slice(), b"XY");
        assert_eq!(haystack.as_slice(), b"YZ");
    }
}

pub fn read_line(buf: &mut BufReader<TcpStream>) -> Result<String, Error> {
    read_until(buf, CRLF)
}

fn read_until(buf: &mut BufReader<TcpStream>, terminator: &str) -> Result<String, Error> {
    let delay = time::Duration::from_millis(5);
    read_until_with_delay(buf, terminator, delay)
}

// TODO - make this properly handle the terminator param
fn read_until_with_delay(buf: &mut BufReader<TcpStream>, terminator: &str, delay: Duration) -> Result<String, Error> {
    // this only works becaue buf.read_line stops on a "\n" character
    let mut line = String::new();

    loop {
        let mut next_str = String::new();
        buf.read_line(&mut next_str).expect("should be able to read from stream");
        line.push_str(next_str.as_str());
        if line.ends_with(terminator) {
            break;
        } else {
            // not all data is available in the input stream.
            // sleep before trying again to avoid wasting CPU cycles
            thread::sleep(delay);
        }
    }

    Ok(line)
}
