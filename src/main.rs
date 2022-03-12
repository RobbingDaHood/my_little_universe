use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

mod time;
mod gameloop;
mod products;
mod station;
mod external_commands;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                // connection succeeded
                let mut buffer = [0; 1024];
                let buffer_size = stream.read(&mut buffer).unwrap();
                println!("Recived {:?}", String::from_utf8_lossy(&buffer[..buffer_size]));

                stream.write("Done \n".as_bytes());
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }

    // close the socket server
    drop(listener);
}
