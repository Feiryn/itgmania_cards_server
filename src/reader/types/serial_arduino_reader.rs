use std::collections::HashMap;

use tokio::{io::AsyncReadExt, task::JoinHandle};
use tokio_serial::SerialPortBuilderExt;

use crate::{
    accounts::check_account_exists,
    cards::{
        card_type::CardType,
        cards_manager::{set_current_card_number_player1, set_current_card_number_player2},
    },
    reader::reader::{ReaderTrait, StartReaderResult},
};

pub struct SerialArduinoReader {
    serial_thread: Option<JoinHandle<()>>,
}

impl SerialArduinoReader {
    pub fn new() -> Self {
        SerialArduinoReader {
            serial_thread: None,
        }
    }
}

async fn read_and_insert_card(reader_number: u8, split_data: &[&str]) -> Result<(), String> {
    if split_data.len() != 3 {
        return Err(format!(
            "Invalid data format for CARD_{}: {}",
            reader_number,
            split_data.join(" ")
        ));
    }

    let card_type = CardType::try_from(split_data[1].to_string())?;

    let card_id = split_data[2].to_string();

    if !check_account_exists(&card_type, &card_id) {
        return Err(format!(
            "Card {}_{} does not exist",
            card_type.to_string(),
            card_id
        ));
    }

    if reader_number == 1 {
        set_current_card_number_player1(card_type, card_id).await;
    } else if reader_number == 2 {
        set_current_card_number_player2(card_type, card_id).await;
    } else {
        return Err(format!("Unknown reader number: {}", reader_number));
    }

    Ok(())
}

impl ReaderTrait for SerialArduinoReader {
    fn start(&mut self, config: HashMap<String, String>) -> StartReaderResult {
        let serial_path = config.get("serial_port").expect("Serial port not specified in config. For Linux, it should be something like /dev/serial/by-id/usb-Arduino__www.arduino.cc__[...]. For Windows, it should be something like COM3.").to_owned();

        // Start the serial reading thread
        let serial_thread = tokio::spawn(async move {
            let mut serial_port = tokio_serial::new(serial_path, 115200)
                .open_native_async()
                .expect("Failed to open serial port");

            // Read from the serial port in a loop
            let mut buffer = [0u8; 1024];
            let mut line_buffer = String::new();
            let mut cumulative_errors = 0;
            loop {
                match serial_port.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        line_buffer.push_str(&String::from_utf8_lossy(&buffer[..n]));

                        // Process every complete line (terminated by '\n')
                        while let Some(newline_pos) = line_buffer.find('\n') {
                            let line = line_buffer[..newline_pos].trim().to_string();
                            line_buffer = line_buffer[newline_pos + 1..].to_string();

                            if line.is_empty() {
                                continue;
                            }

                            let split_data: Vec<&str> = line.split(' ').collect();

                            match split_data[0] {
                                "CARD_1" => {
                                    if let Err(e) = read_and_insert_card(1, &split_data).await {
                                        eprintln!("Error processing CARD_1 data: {}", e);
                                    }
                                }
                                "CARD_2" => {
                                    if let Err(e) = read_and_insert_card(2, &split_data).await {
                                        eprintln!("Error processing CARD_2 data: {}", e);
                                    }
                                }
                                "READER_1_FOUND" => {
                                    println!("Reader 1 found");
                                }
                                "READER_2_FOUND" => {
                                    println!("Reader 2 found");
                                }
                                "STARTING" => {
                                    println!("Starting arduino...");
                                }
                                "NO_CARD_1" => {}
                                "NO_CARD_2" => {}
                                _ => {
                                    eprintln!("Unknown data received from serial port: {}", line);
                                }
                            }
                        }
                    }
                    Ok(_) => {} // No data read, continue the loop
                    Err(e) => {
                        eprintln!("Error reading from serial port: {}", e);
                        cumulative_errors += 1;
                        if cumulative_errors >= 5 {
                            eprintln!("Too many errors reading from serial port. Stopping reader.");
                            break;
                        }
                    }
                }
            }
        });

        self.serial_thread = Some(serial_thread);

        StartReaderResult {
            must_pull_cards: false,
        }
    }

    fn pull_cards(&self) {
        // No need to pull cards, as the reader will push card data to the server when a card is read on the serial port.
    }
}
