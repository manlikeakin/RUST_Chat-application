
use std::net::SocketAddr;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .await
        .expect("Could not bind to port 7878");

    let (tx, _) = broadcast::channel::<String>(100);

    println!("Chat server listening on 127.0.0.1:7878");

    loop {
        let (socket, address) = listener
            .accept()
            .await
            .expect("Could not accept client");

        println!("Client connected: {address}");

        let tx = tx.clone();

        tokio::spawn(async move {
            handle_client(socket, address, tx).await;
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    address: SocketAddr,
    tx: broadcast::Sender<String>,
) {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    // Step 1: Read the username when the client connects.
    let mut username = String::new();

    match reader.read_line(&mut username).await {
        Ok(0) => {
            println!("Client disconnected before sending a username: {address}");
            return;
        }
        Ok(_) => {}
        Err(error) => {
            eprintln!("Could not read username from {address}: {error}");
            return;
        }
    }

    let username = username.trim().to_string();

    // Step 2: Make sure the username isn't empty.
    if username.is_empty() {
        let _ = writer.write_all(b"Username cannot be empty.\n").await;
        println!("Rejected empty username from {address}");
        return;
    }

    println!("{username} joined from {address}");

    // Step 3: Subscribe this client to broadcast messages.
    let mut rx = tx.subscribe();

    // Step 4: Announce that this user has joined.
    let joined_message = format!("*** {username} joined the chat ***\n");
    let _ = tx.send(joined_message);

    // Task 1: Send broadcast messages to this client.
    let writer_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(message) => {
                    if writer.write_all(message.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("{address} missed {skipped} messages");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Task 2: Read chat messages from this client and broadcast them.
    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("{username} disconnected");
                break;
            }
            Ok(_) => {
                let message = format!("{username}: {line}");

                if tx.send(message).is_err() {
                    eprintln!("Could not broadcast message from {username}");
                }
            }
            Err(error) => {
                eprintln!("Read error from {username}: {error}");
                break;
            }
        }
    }

    // Step 5: Announce that this user has left.
    let left_message = format!("*** {username} left the chat ***\n");
    let _ = tx.send(left_message);

    // Stop the sending task when this client disconnects.
    writer_task.abort();
}
