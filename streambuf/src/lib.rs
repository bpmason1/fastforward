use std::io::BufReader;
use std::io::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::{self, Duration};

const CRLF: &str = "\r\n";

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
