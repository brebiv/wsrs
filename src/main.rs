use anyhow::{Result, bail};
use base64::{Engine, engine::general_purpose};
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::{self, TcpListener, TcpStream},
};
use wsrs::{Frame, accept_connection, send_bad_request};

const REQUIRED_HEADERS: [&str; 5] = [
    "Host",
    "Upgrade",
    "Connection",
    "Sec-WebSocket-Key",
    "Sec-WebSocket-Version",
];

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    fn build(data: Vec<String>) -> Result<Self> {
        println!("Building HTTP request");

        let first_line_parts: Vec<&str> = data.first().unwrap().split_whitespace().collect();
        if first_line_parts.len() != 3 {
            bail!("Invalid first line");
        }
        let method = first_line_parts[0].to_string();
        let path = first_line_parts[1].to_string();
        let version = first_line_parts[2].to_string();
        let mut headers = HashMap::new();

        let mut required_headers_count: usize = 0;

        for line in data.iter().skip(1) {
            let parts: Vec<&str> = line.split(": ").collect();
            let header = parts[0];

            if REQUIRED_HEADERS.contains(&header) {
                required_headers_count += 1;
            }

            let header_value = parts[1];
            headers.insert(header.to_string(), header_value.to_string());
        }

        if required_headers_count != REQUIRED_HEADERS.len() {
            bail!("Required headers issue");
            // send_bad_request(&mut stream);
            // panic!("Holly shit");
        }

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
        })
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    let (mut stream, client_addr) = listener.accept().unwrap();

    println!("{:#?}, {:#?}", stream, client_addr);

    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    let lines: Vec<String> = buf_reader
        .by_ref()
        .lines()
        .map(|l| l.unwrap())
        .take_while(|l| !l.is_empty())
        .collect();

    let req = HttpRequest::build(lines);
    dbg!(&req);

    if let Ok(req) = req {
        let mut hasher = Sha1::new();
        let secret = format!(
            "{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
            req.headers.get("Sec-WebSocket-Key").unwrap()
        );
        let secret = {
            hasher.update(secret);
            hasher.finalize()
        };
        let secret = general_purpose::STANDARD.encode(secret);
        accept_connection(&mut stream, &secret);
    } else {
        send_bad_request(&mut stream);
        return;
    }

    loop {
        let mut buf = [0u8; 1024];
        match buf_reader.read(&mut buf) {
            Ok(0) => {
                println!("Client is done");
                break;
            }
            Ok(n) => {
                println!("Got {n} bytes");
                let frame = &buf[..n];
                println!("{:#x?}", &frame);
                Frame::parse(&frame);

                let payload = ['a' as u8; 65536 + 10];
                // let payload = "hello bitch!";
                let resp_frame = Frame::new(
                    true, false, false, false, 1, false, &payload,
                    // payload.as_bytes(),
                );
                let resp_bytes = resp_frame.as_bytes();
                // println!("{:08b}", resp_bytes[0]);
                // println!("{:08b}", resp_bytes[1]);
                // for byte in resp_bytes.iter().skip(2) {
                //     println!("{:08b}", byte);
                // }
                stream.write(&resp_bytes).unwrap();
            }
            Err(e) => {
                eprintln!("Got error: {e}");
                break;
            }
        };
    }
}
