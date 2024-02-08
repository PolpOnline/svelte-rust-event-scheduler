//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/server.rs
//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/data.rs
//https://github.com/hyperium/tonic/blob/master/examples/proto/routeguide/route_guide.proto

use crate::g_rpc::event_scheduler::schedule_service_server::ScheduleServiceServer;
use crate::g_rpc::MyScheduleService;
use tonic::transport::Server;

pub mod g_rpc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let schedule_service = MyScheduleService::default();

    println!("Service listening on {}", addr);

    Server::builder()
        .add_service(ScheduleServiceServer::new(schedule_service))
        .serve(addr)
        .await?;

    Ok(())
}
