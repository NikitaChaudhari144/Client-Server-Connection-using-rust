use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// Define constants for the local address and message size
const LOCAL: &str = "127.0.0.1:1895";
const MSG_SIZE: usize = 512;

// Function to format a message string into a HashMap
fn format_msg(msg: String) -> HashMap<String, String> {
    // Split the message string into lines
    let lines: Vec<&str> = msg.lines().collect();

    // Create a new HashMap to store the formatted message
    let mut res_hm: HashMap<String, String> = HashMap::new();

    // Extract the operation, key length, and key from the message
    let operation = lines[0];
    let key_len_line = lines[1];
    let key_line = lines[2];

    // Parse the key length and key from their respective lines
    let key_len: usize = key_len_line
        .split(':')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let key = key_line.split(':').nth(1).unwrap_or("");

    // If the operation is PUT and the message has 5 lines, extract the value length and value
    if operation == "PUT" && lines.len() == 5 {
        let val_len_line = lines[3];
        let val_line = lines[4];

        let val_len: usize = val_len_line
            .split(':')
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let value = val_line.split(':').nth(1).unwrap_or("");

        // Insert the value length and value into the HashMap
        res_hm.insert("val_len".to_string(), val_len.to_string());
        res_hm.insert("value".to_string(), value.to_string());
    }

    // Insert the operation, key length, and key into the HashMap
    res_hm.insert("operation".to_string(), operation.to_string());
    res_hm.insert("key_len".to_string(), key_len.to_string());
    res_hm.insert("key".to_string(), key.to_string());

    return res_hm;
}

fn main() {
    // Bind the server to the local address
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");

    // Create a new HashMap to store the database
    let database: HashMap<i32, String> = HashMap::new();

    // Create a new Arc-wrapped Mutex to allow for concurrent access to the database
    let rc_database = Arc::new(Mutex::new(database));
    let rc_database_clone = rc_database.clone();

    let mut clients = vec![];

    // Create a channel for sending and receiving messages
    let (tx, rx) = mpsc::channel::<String>();

    // Loop indefinitely to accept new client connections and handle messages
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));
            let rc_database_2 = rc_database_clone.clone();

            // Spawn a new thread to handle the client connection
            thread::spawn(move || loop {
                // Read the message from the client
                let mut buff = vec![0; MSG_SIZE];
                let value_found: String;
                let mut response: String = String::new();

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        // Convert the message buffer to a string and format it as a HashMap
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        let mut clone_instr: HashMap<String, String> = format_msg(msg.clone());

                        // Handle the message based on the operation
                        if clone_instr["operation"] == "PUT" && clone_instr.len() == 5 {
                            let new_key: i32 = clone_instr["key"].parse::<i32>().unwrap();
                            let new_value: String = clone_instr["value"].parse::<String>().unwrap();
                            let mut locked_database = rc_database_2.lock().unwrap();
                            locked_database.insert(new_key, new_value);
                            response = "Result for PUT: OK".to_string();

                            clone_instr.clear();
                        } else if clone_instr["operation"] == "GET" && clone_instr.len() == 3 {
                            let new_key: i32 = clone_instr["key"].parse::<i32>().unwrap();
                            let locked_database = rc_database_2.lock().unwrap();

                            if locked_database.contains_key(&new_key) {
                                value_found = locked_database[&new_key].clone().to_string();
                                response = "Result for GET: OK".to_owned()
                                    + "\n"
                                    + "RESULT-LEN:"
                                    + &value_found.len().to_string()
                                    + "\n"
                                    + "VAL:"
                                    + &value_found
                                    + "\n";
                            } else {
                                response = "Result for GET: ERROR".to_string();
                            }

                            clone_instr.clear();
                        } else if clone_instr["operation"] == "DEL" && clone_instr.len() == 3 {
                            let new_key: i32 = clone_instr["key"].parse::<i32>().unwrap();
                            let mut locked_database = rc_database_2.lock().unwrap();

                            if locked_database.contains_key(&new_key) {
                                locked_database.remove(&new_key);
                                response = "Result for DEL: OK".to_string();
                            } else {
                                response = "Result for DEL: ERROR".to_string();
                            }
                        }

                        println!("\n{}: {:?}", addr, msg);
                        println!("db: {:?}", rc_database_2);
                        println!("-------------------------------\n");

                        // Send the response message to the channel receiver
                        tx.send(response.to_string())
                            .expect("failed to send msg to rx");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("closing connection with: {}", addr);
                        break;
                    }
                }
            });
        }

        // Send messages to all connected clients
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }
    }
}
