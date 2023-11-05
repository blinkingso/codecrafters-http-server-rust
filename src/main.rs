// Uncomment this block to pass the first stage
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use anyhow::Result;

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let buf = BufReader::new(&mut stream);
                let lines: Vec<String> = buf
                    .lines()
                    .map(|b| b.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();
                println!("{:?}", lines);
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;
                stream.flush()?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
