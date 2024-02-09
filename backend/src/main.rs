//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/server.rs
//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/data.rs
//https://github.com/hyperium/tonic/blob/master/examples/proto/routeguide/route_guide.proto

use color_eyre::eyre::Result;

use crate::g_rpc::start_server;

pub mod g_rpc;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    start_server().await?;

    Ok(())
}
