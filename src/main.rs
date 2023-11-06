use std::collections::HashMap;
// Uncomment this block to pass the first stage
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

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
                std::thread::spawn(move || {
                    handle_client_request(stream);
                });
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
            "/user-agent" => {
                let headers = parse_headers(&lines);
                let user_agent = headers.get("User-Agent").unwrap();
                let bytes = user_agent.as_bytes().len();
                stream.write(b"HTTP/1.1 200 OK\r\n").unwrap();
                stream.write(b"Content-Type: text/plain\r\n").unwrap();
                stream
                    .write(format!("Content-Length: {}\r\n\r\n", bytes).as_bytes())
                    .unwrap();
                stream
                    .write(format!("{}\r\n", user_agent).as_bytes())
                    .unwrap();
            }
            path => {
                if path.starts_with("/echo/") && method == HttpMethod::Get {
                    let response_content = path.strip_prefix("/echo/").unwrap_or("");
                    let bytes = response_content.len();
                    stream.write(b"HTTP/1.1 200 OK\r\n").unwrap();
                    stream.write(b"Content-Type: text/plain\r\n").unwrap();
                    stream
                        .write(format!("Content-Length: {}\r\n\r\n", bytes).as_bytes())
                        .unwrap();
                    stream
                        .write(format!("{}\r\n", response_content).as_bytes())
                        .unwrap();
                } else if path.starts_with("/files/") {
                    let mut args = std::env::args();
                    let dir = args
                        .nth(2)
                        .and_then(|s| Some(s.trim().to_string()))
                        .unwrap_or_default();
                    let filename = path.strip_prefix("/files/").and_then(|file| {
                        let mut path = Path::new(dir.as_str()).to_path_buf();
                        path.push(file);
                        println!("path: {:?}", path);
                        std::fs::read(path).ok()
                    });
                    if let Some(file) = filename {
                        stream.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
                        stream
                            .write_all(b"Content-Type: application/octet-stream\r\n")
                            .unwrap();
                        stream
                            .write_all(format!("Content-Length: {}\r\n\r\n", file.len()).as_bytes())
                            .unwrap();

                        stream.write_all(&file).unwrap();
                        stream.write_all("\r\n".as_bytes()).unwrap();
                    }
                } else {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            }
        }
        stream.flush().unwrap();
    }
}

fn parse_headers(headers: &Vec<String>) -> HashMap<&str, &str> {
    headers
        .iter()
        .skip(1)
        .filter(|s| !s.trim().is_empty())
        .map(|header| {
            let (key, value) = header.split_once(":").unwrap();
            (key.trim(), value.trim())
        })
        .collect::<HashMap<&str, &str>>()
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
