// Uncomment this block to pass the first stage
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Result;

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
        let path = line.next().unwrap();
        match path {
            "/" => {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
            }
            _ => {
                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
            }
        }
        stream.flush().unwrap();
    }
}
