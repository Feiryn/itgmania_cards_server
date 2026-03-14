use std::collections::HashMap;

use crate::reader::types::serial_arduino_reader::SerialArduinoReader;

pub struct StartReaderResult {
    // Indicates whether the reader requires periodic pulling of cards (e.g., for readers that don't push card data automatically)
    pub must_pull_cards: bool,
}

pub trait ReaderTrait: Send + Sync {
    fn start(&mut self, config: HashMap<String, String>) -> StartReaderResult;
    fn pull_cards(&self) -> ();
}

pub fn start_reader() {
    // Read the configuration for the reader from the config file
    let config = toml::from_str::<HashMap<String, String>>(
        &std::fs::read_to_string("reader.toml").expect("Failed to read reader configuration file"),
    )
    .expect("Failed to parse reader configuration file");

    // Create the reader from the type specified in the configuration
    let reader_type = config
        .get("type")
        .expect("Reader type not specified in config")
        .to_owned();

    let mut reader: Box<dyn ReaderTrait> = match reader_type.as_str() {
        "serial_arduino" => Box::new(SerialArduinoReader::new()),
        "none" => {
            println!("No reader type specified, skipping reader initialization");
            return;
        }
        _ => panic!("Unknown reader type specified in config: {}", reader_type),
    };

    // Start the reader
    let start_result = reader.start(config);

    // Start a thread to pull cards from the reader every 250 milliseconds if the reader requires it
    if start_result.must_pull_cards {
        std::thread::spawn(move || {
            loop {
                reader.pull_cards();
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });
    }
}
