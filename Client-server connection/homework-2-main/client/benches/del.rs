extern crate criterion;
use criterion::Criterion;
//use rand::Rng;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

// Define constants for the local address and message size
const LOCAL: &str = "127.0.0.1:1895";
const MSG_SIZE: usize = 512;

// Function to automatically generate a DELETE request
fn generate_del_req() -> String {
    let key = 2;

    // Construct the DELETE request string
    let del_req = "DEL\nKEY-LEN:".to_owned()
        + &key.to_string().len().to_string()
        + "\nKEY:"
        + &key.to_string()
        + "\n";

    return del_req;
}

// Benchmark function for DELETE requests
fn bench_del_req(b: &mut Criterion) {
    // Define the benchmark function
    b.bench_function("Bench Delete Request", |c| {
        c.iter(|| {
            // Connect to the local server
            let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
            client
                .set_nonblocking(true)
                .expect("failed to initiate non-blocking");

            // Create a channel for sending and receiving messages
            let (tx, rx) = mpsc::channel::<String>();

            // Spawn a new thread to handle the client connection
            thread::spawn(move || {
                // Read the message from the server
                let mut buff = vec![0; MSG_SIZE];
                match client.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("connection with server was severed");
                    }
                }

                // Send the message to the server
                match rx.try_recv() {
                    Ok(msg) => {
                        let mut buff = msg.clone().into_bytes();
                        buff.resize(MSG_SIZE, 0);
                        client.write_all(&buff).expect("writing to socket failed");
                    }
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => (),
                }
            });

            // Generate the DELETE request message
            let mut buff = generate_del_req();

            // Trim the message and send it to the server
            let msg = buff.trim().to_string();
            if tx.send(msg).is_err() {
                println!("Quitting");
            }
        });
    });
}

criterion::criterion_group!(benches, bench_del_req);
criterion::criterion_main!(benches);
