use crate::cards::cards_manager;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

const SOCKET_PATH: &str = "/tmp/itgmania_cards.sock";

pub async fn run_unix_socket_server() {
    // Remove existing socket file if it exists
    let socket_path = PathBuf::from(SOCKET_PATH);
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            eprintln!("Failed to remove existing socket file: {}", e);
            return;
        }
    }

    // Create the Unix socket listener
    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(listener) => {
            println!("Unix socket server listening on: {}", SOCKET_PATH);
            listener
        }
        Err(e) => {
            eprintln!("Failed to bind Unix socket: {}", e);
            return;
        }
    };

    // Accept connections in a loop
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(stream: UnixStream) {
    let mut reader = BufReader::new(stream);

    // Keep reading commands until the client disconnects
    loop {
        let mut command = String::new();

        // Read command from client
        match reader.read_line(&mut command).await {
            Ok(0) => {
                // Connection closed
                return;
            }
            Ok(_) => {
                let command = command.trim();

                // Handle commands
                if command == "READ" {
                    // Get both player cards
                    let player1_card = cards_manager::get_current_card_number_player1().await;
                    let player2_card = cards_manager::get_current_card_number_player2().await;

                    // Send the response
                    let response = player1_card.map_or_else(
                        || "0".to_string(),
                        |(card_type, card_number)| {
                            format!("{}_{}", card_type.to_string(), card_number)
                        },
                    ) + ","
                        + &player2_card.map_or_else(
                            || "0".to_string(),
                            |(card_type, card_number)| {
                                format!("{}_{}", card_type.to_string(), card_number)
                            },
                        )
                        + "\n";

                    if let Err(e) = reader.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write to socket: {}", e);
                        return;
                    }
                } else if command.starts_with("RESET ") {
                    // Parse player number
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    if parts.len() == 2 {
                        match parts[1] {
                            "1" => {
                                cards_manager::clear_current_card_player1().await;
                                let response = "OK\n";
                                if let Err(e) = reader.write_all(response.as_bytes()).await {
                                    eprintln!("Failed to write to socket: {}", e);
                                    return;
                                }
                            }
                            "2" => {
                                cards_manager::clear_current_card_player2().await;
                                let response = "OK\n";
                                if let Err(e) = reader.write_all(response.as_bytes()).await {
                                    eprintln!("Failed to write to socket: {}", e);
                                    return;
                                }
                            }
                            _ => {
                                let response = "ERROR: Invalid player number (must be 1 or 2)\n";
                                if let Err(e) = reader.write_all(response.as_bytes()).await {
                                    eprintln!("Failed to write to socket: {}", e);
                                    return;
                                }
                            }
                        }
                    } else {
                        let response = "ERROR: Invalid RESET command format (use: RESET <1|2>)\n";
                        if let Err(e) = reader.write_all(response.as_bytes()).await {
                            eprintln!("Failed to write to socket: {}", e);
                            return;
                        }
                    }
                } else if command == "ENABLE" {
                    cards_manager::set_enabled(true).await;
                    let response = "OK\n";
                    if let Err(e) = reader.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write to socket: {}", e);
                        return;
                    }
                } else if command == "DISABLE" {
                    cards_manager::set_enabled(false).await;
                    let response = "OK\n";
                    if let Err(e) = reader.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write to socket: {}", e);
                        return;
                    }
                } else {
                    let response =
                        "ERROR: Unknown command (use: READ, RESET <1|2>, ENABLE, or DISABLE)\n";
                    if let Err(e) = reader.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write to socket: {}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                return;
            }
        }
    }
}
