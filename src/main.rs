use crate::reader::reader::start_reader;

mod accounts;
mod auth;
mod cards;
mod config;
mod reader;
mod socket_server;
mod templates;
mod web_server;

#[tokio::main]
async fn main() {
    // Start the Unix socket server in a separate task
    tokio::spawn(async {
        socket_server::run_unix_socket_server().await;
    });

    // Start the card reader (a separate task will be spawned inside the start function)
    start_reader();

    // Start the web server (this will block until shutdown)
    let _ = web_server::build_rocket().launch().await;
}
