//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/server.rs
//https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/data.rs
//https://github.com/hyperium/tonic/blob/master/examples/proto/routeguide/route_guide.proto

use crate::event_scheduler::{
    PingReply, PingRequest, SubscriberCountRequest, SubscriberCountStreamUpdate,
};
use event_scheduler::schedule_service_server::ScheduleServiceServer;
use mpsc::Receiver;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

use crate::event_scheduler::schedule_service_server::ScheduleService;

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<SubscriberCountStreamUpdate, Status>> + Send>>;

pub mod event_scheduler {
    tonic::include_proto!("online.polp.schedule_service");
}

type SubscribersToNotify = Vec<Sender<SubscriberCountStreamUpdate>>;

#[derive(Default)]
pub struct MyScheduleService {
    subscribers: Arc<Mutex<SubscribersToNotify>>,
}

impl MyScheduleService {
    async fn notify_subscribers_task(
        &self,
        mut update: Receiver<SubscriberCountStreamUpdate>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let update = update.recv().await.ok_or(
                "Unable to receive SubscriberCountStreamUpdate, the channel has been closed",
            )?;
            let subscribers = self.subscribers.lock().await;

            // TODO: make this a parallel operation
            for subscriber in subscribers.iter() {
                subscriber.send(update.clone()).await?;
            }
        }
    }
}

#[tonic::async_trait]
impl ScheduleService for MyScheduleService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingReply>, Status> {
        println!("Got a ping from {:?}", request.remote_addr());

        let reply = PingReply {
            message: "Pong!".to_string(),
        };
        Ok(Response::new(reply))
    }

    type SubscriberCountStream = ResponseStream;

    async fn subscriber_count(
        &self,
        request: Request<SubscriberCountRequest>,
    ) -> Result<Response<Self::SubscriberCountStream>, Status> {
        println!(
            "Got a subscriber count streaming request from {:?}",
            request.remote_addr()
        );

        let (tx, rx) = mpsc::channel(4);

        self.subscribers.lock().await.push(tx);

        let output_stream = ReceiverStream::new(rx).map(Ok::<_, Status>);
        Ok(Response::new(
            Box::pin(output_stream) as Self::SubscriberCountStream
        ))
    }
}

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
