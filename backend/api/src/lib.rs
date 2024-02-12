// mod auth;
mod grpc;

use grpc::start_server;

#[tokio::main]
pub async fn main() {
    dotenvy::dotenv().ok();

    start_server().await.expect("Failed to start the server");
}
