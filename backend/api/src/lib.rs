mod grpc;

use color_eyre::Result;
use grpc::start_server;
use migration::sea_orm::Database;
use std::env;

#[tokio::main]
pub async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL")?;
    let db = Database::connect(db_url).await?;

    // [Insert, Select, Update, Delete operations here]
    start_server().await?;

    Ok(())
}
