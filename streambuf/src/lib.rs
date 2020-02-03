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

    pub fn read_once_until(&mut self, terminator: &[u8]) -> Option<Vec<u8>> {
        self.read_inner_to_buffer();
        
        if self.buffer.len() < terminator.len() {
            return None;
        }

        let start_idx_opt = find_subsequence(self.buffer.as_slice(), terminator);
        let result: Option<Vec<u8>> = match start_idx_opt {
            Some(start_idx) => {
                let end_idx = start_idx + terminator.len();
                let data: Vec<u8> = self.buffer.drain(..end_idx).collect();
                Some(data)
            },
            None => None
        };
        result
    }

}


fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    twoway::find_bytes(haystack, needle)
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
