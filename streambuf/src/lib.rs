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
    
    let stream = StreamBuf{
        inner: inner,
        buffer: Vec::with_capacity(100),
    };
    let timeout  = Duration::from_millis(1);
    stream.inner.set_read_timeout(Some(timeout));
    stream
}

impl StreamBuf {
    fn read_inner_to_buffer(&mut self) { // -> Result<usize, Error> {
        let mut buffer = [0; 100];
        let result = self.inner.read(&mut buffer[..]);
        // println!("{}", std::str::from_utf8(&buffer).unwrap());

        if result.is_ok() {
            let num_bytes = result.unwrap();
            // self.buffer.append(&mut buffer);
            for idx in 0..num_bytes {
                self.buffer.push(buffer[idx]);
            }
        }
        
        // let mut tmp_vec = Vec::with_capacity(100);
        // let result = self.inner.read( &mut tmp_vec );
        // println!("{:?}", tmp_vec);
        // if result.is_ok() {
        //     self.buffer.append(&mut tmp_vec);
        // }
        // result
    }

    pub fn read_num_bytes(&mut self, cnt: usize) -> Option<Vec<u8>> {
        let delay = Duration::from_millis(5);

        self.read_inner_to_buffer();

        while self.buffer.len() < cnt {
            // not all data is available in the input stream.
            // sleep before trying again to avoid wasting CPU cycles
            // thread::sleep(delay);
        }
        let data: Vec<u8> = self.buffer.drain(..cnt).collect();
        Some(data)
    }

    pub fn read_line(&mut self) -> Option<Vec<u8>> {
        self.read_until(b"\r\n")
    }

    // TODO - add timeout
    pub fn read_until(&mut self, terminator: &[u8]) -> Option<Vec<u8>> {
        let delay = Duration::from_millis(5);

        let mut result = None;
        while result.is_none() {
            result = self.read_once_until(terminator);
            // thread::sleep(delay);
        }
        result
    }

    fn read_once_until(&mut self, terminator: &[u8]) -> Option<Vec<u8>> {
        self.read_inner_to_buffer();
        let result = read_once_until(&mut self.buffer, terminator);
        result
    }

}

fn read_once_until(haystack: &mut Vec<u8>, needle: &[u8]) -> Option<Vec<u8>> {
    if haystack.len() < needle.len() {
        return None;
    }

    let start_idx_opt = twoway::find_bytes(haystack.as_slice(), needle);
    // println!("{:?}start_idx_opt", start_idx_opt);
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
