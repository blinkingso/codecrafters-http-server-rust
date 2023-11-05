// Uncomment this block to pass the first stage
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::{Error, Result};

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client_request(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client_request(mut stream: TcpStream) {
    let buf = BufReader::new(&mut stream);
    let lines: Vec<String> = buf
        .lines()
        .map(|b| b.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    if let Some(header) = lines.get(0) {
        let mut line = header.split_whitespace();
        let _method = line.next().unwrap();
        let method = HttpMethod::try_from(_method).unwrap();
        let path = line.next().unwrap();
        match path {
            "/" => {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
            }
            path => {
                if path.starts_with("/echo/") && method == HttpMethod::Get {
                    let response_content = path.strip_prefix("/echo").unwrap_or("");
                    let bytes = response_content.len();
                    stream.write(b"HTTP/1.1 200 OK\r\n").unwrap();
                    stream.write(b"Content-Type: text/plain\r\n").unwrap();
                    stream
                        .write(format!("Content-Length: {}\r\n\r\n", bytes).as_bytes())
                        .unwrap();
                    stream
                        .write(format!("{}\r\n", response_content).as_bytes())
                        .unwrap();
                } else {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            }
        }
        stream.flush().unwrap();
    }
}

#[derive(PartialEq, Eq)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Trace,
}

impl TryFrom<&str> for HttpMethod {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Ok(match value.as_bytes() {
            b"GET" => HttpMethod::Get,
            b"POST" => HttpMethod::Post,
            b"PUT" => HttpMethod::Put,
            b"TRACE" => HttpMethod::Trace,
            b"DELETE" => HttpMethod::Delete,
            _ => return Err(Error::msg("Method not supported!")),
        })
    }
}
