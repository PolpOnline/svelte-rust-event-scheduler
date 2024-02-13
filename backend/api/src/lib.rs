use grpc::start_server;

// mod auth;
// mod auth;
mod grpc;

#[tokio::main]
pub async fn main() {
    dotenvy::dotenv().ok();

    start_server().await.expect("Failed to start the server");
}
