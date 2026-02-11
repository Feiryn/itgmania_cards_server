mod auth;
mod config;
mod templates;
mod web_server;
mod accounts;
mod cards_manager;
mod socket_server;

#[tokio::main]
async fn main() {
    // Start the Unix socket server in a separate task
    tokio::spawn(async {
        socket_server::run_unix_socket_server().await;
    });

    // Start the web server (this will block until shutdown)
    let _ = web_server::build_rocket().launch().await;
}