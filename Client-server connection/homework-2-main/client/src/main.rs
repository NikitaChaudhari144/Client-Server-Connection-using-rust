use rand::Rng;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

// Define constants for the local address, message size, and key range
const LOCAL: &str = "127.0.0.1:1895";
const MSG_SIZE: usize = 512;
const KEY_RANGE: i32 = 5;

// Define constants for the minimum and maximum number of words in a PUT request
const MIN_NUM_OF_WORDS: i32 = 5;
const MAX_NUM_OF_WORDS: i32 = 10;

// Function to generate a PUT request
fn generate_put_req() -> String {
    let mut rng = rand::thread_rng();

    // Generate a random key and value for the PUT request
    let key = rng.gen_range(0..KEY_RANGE);
    let words = ["apple", "mango", "banana", "guava", "peach", "kiwi"];
    let num_of_words = rng.gen_range(MIN_NUM_OF_WORDS..MAX_NUM_OF_WORDS);
    let mut value = String::new();

    // Construct the value string by concatenating random words
    for _i in 0..num_of_words {
        let word = words[rng.gen_range(0..words.len())];
        value += &(word.to_owned() + " ");
    }

    // Construct the PUT request string
    let put_req = "PUT\nKEY-LEN:".to_owned()
        + &key.to_string().len().to_string()
        + "\nKEY:"
        + &key.to_string()
        + "\nVAL-LEN:"
        + &value.to_string().len().to_string()
        + "\nVAL:"
        + &value
        + "\n";

    return put_req;
}

// Function to generate a GET request
fn generate_get_req() -> String {
    let mut rng = rand::thread_rng();

    // Generate a random key for the GET request
    let key = rng.gen_range(0..KEY_RANGE);

    // Construct the GET request string
    let get_req = "GET\nKEY-LEN:".to_owned()
        + &key.to_string().len().to_string()
        + "\nKEY:"
        + &key.to_string()
        + "\n";

    return get_req;
}

// Function to generate a DELETE request
fn generate_del_req() -> String {
    let mut rng = rand::thread_rng();

    // Generate a random key for the DELETE request
    let key = rng.gen_range(0..KEY_RANGE);

    // Construct the DELETE request string
    let del_req = "DEL\nKEY-LEN:".to_owned()
        + &key.to_string().len().to_string()
        + "\nKEY:"
        + &key.to_string()
        + "\n";

    return del_req;
}

fn main() {
    // Connect to the local server
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    // Create a channel for sending and receiving messages
    let (tx, rx) = mpsc::channel::<String>();

    // Spawn a new thread to handle the client connection
    thread::spawn(move || loop {
        // Read the message from the server
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                println!("{}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
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
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(1));
    });

    // Generate and send random requests to the server
    loop {
        let mut buff = String::new();
        let rand_req = rand::thread_rng().gen_range(0..3);

        match rand_req {
            0 => buff = generate_put_req(),
            1 => buff = generate_get_req(),
            2 => buff = generate_del_req(),
            _ => (),
        }

        let msg = buff.trim().to_string();
        if tx.send(msg).is_err() {
            break;
        }

        thread::sleep(Duration::from_millis(1));
    }

    println!("Client closed!");
}
