//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/server.rs
//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/data.rs
//https://github.com/hyperium/tonic/blob/master/examples/proto/routeguide/route_guide.proto

use crate::g_rpc::event_scheduler::schedule_service_server::ScheduleServiceServer;
use crate::g_rpc::MyScheduleService;
use tonic::codegen::CompressionEncoding;
use tonic::transport::Server;

mod db;
pub mod g_rpc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GET the address to listen on from an environment variable
    let addr = std::env::var("ADDRESS")?.parse()?;

    let schedule_service = MyScheduleService::default();

    println!("Service listening on {}", addr);

    let schedule_service_server = ScheduleServiceServer::new(schedule_service)
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd);

    Server::builder()
        .add_service(schedule_service_server)
        .serve(addr)
        .await?;

    Ok(())
}
