use std::{io::Write, net::TcpStream};

pub struct Frame {
    fin: bool,
    rsv1: bool,
    rsv2: bool,
    rsv3: bool,
    opcode: u8,
    mask: bool,
    payload_length: u8,
    payload: [u8],
}

impl Frame {
    pub fn build(data: &[u8]) {
        println!("First byte: {:08b}", data[0]);

        println!("Fin: {:08b}", (data[0] >> 7) & 0x01);
        println!("RSV 1: {:08b}", (data[0] >> 6) & 0b00000001);
        println!("RSV 2: {:08b}", (data[0] >> 5) & 0b00000001);
        println!("RSV 3: {:08b}", (data[0] >> 4) & 0b00000001);
        println!("Opcode: {:08b}", (data[0]) & 0b00001111);

        println!();

        println!("Second byte: {:08b}", data[1]);
        let mask = (data[1] >> 7) & 0b00000001;
        let mut mask_key_offset = 2;

        let payload_length = match data[1] & 0b01111111 {
            len @ 0..=125 => len as usize,
            126 => {
                mask_key_offset = 4;
                u16::from_be_bytes(data[2..4].try_into().unwrap()) as usize
            }
            127 => {
                mask_key_offset = 6;
                u32::from_be_bytes(data[2..6].try_into().unwrap()) as usize
            }
            _ => unreachable!("Should never be here"),
        };
        println!("payload length: {payload_length}");

        // let mask_key = &data[2..6];
        let mask_key = &data[mask_key_offset..mask_key_offset + 4];

        let payload = &data[mask_key_offset + 4..];
        let mut decoded = Vec::with_capacity(payload.len());

        for (i, b) in payload.iter().enumerate() {
            let key = mask_key[i % 4];
            decoded.push(b ^ key);
        }

        println!("Payload: {:#?}", str::from_utf8(&decoded));
    }
}

pub fn send_bad_request(stream: &mut TcpStream) {
    let mut resp = Vec::new();
    resp.push("HTTP/1.1 400 Bad Request\r\n".to_string());
    resp.push("\r\n".to_string());

    let resp = resp.join("").into_bytes();

    println!("Sending response...");
    println!("{}", String::from_utf8(resp.clone()).unwrap());

    stream.write(resp.as_slice());
}

pub fn accept_connection(stream: &mut TcpStream, secret: &String) {
    let mut resp = Vec::new();
    resp.push("HTTP/1.1 101 Switching Protocols\r\n".to_string());
    resp.push("Upgrade: websocket\r\n".to_string());
    resp.push("Connection: Upgrade\r\n".to_string());
    resp.push(format!("Sec-WebSocket-Accept: {secret}\r\n").to_string());
    resp.push("Sec-WebSocket-Protocol: chat\r\n".to_string());
    resp.push("\r\n".to_string());

    let resp = resp.join("").into_bytes();

    println!("Sending response...");
    println!("{}", String::from_utf8(resp.clone()).unwrap());

    stream.write(resp.as_slice());
}
