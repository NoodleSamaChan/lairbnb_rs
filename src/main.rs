use std::net::TcpListener;

use lairbnb_rs::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");
    run(listener)?.await
}
