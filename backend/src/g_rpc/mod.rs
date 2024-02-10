use std::pin::Pin;
use std::sync::Arc;

use color_eyre::eyre::{eyre, Result};
use event_scheduler::schedule_service_server::ScheduleService;
use futures::future::join_all;
use mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::codec::CompressionEncoding;
use tonic::codegen::tokio_stream;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::g_rpc::event_scheduler::schedule_service_server::ScheduleServiceServer;

pub async fn start_server() -> Result<()> {
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

type ResponseStream = Pin<
    Box<dyn Stream<Item = Result<event_scheduler::SubscriberCountStreamUpdate, Status>> + Send>,
>;

pub mod event_scheduler {
    tonic::include_proto!("online.polp.schedule_service");
}

type SubscribersToNotify = Vec<Sender<event_scheduler::SubscriberCountStreamUpdate>>;

#[derive(Default)]
pub struct MyScheduleService {
    subscribers: Arc<Mutex<SubscribersToNotify>>,
}

impl MyScheduleService {
    async fn notify_subscribers_task(
        &self,
        mut update: Receiver<event_scheduler::SubscriberCountStreamUpdate>,
    ) -> Result<()> {
        loop {
            let update = update.recv().await.ok_or(eyre!(
                "Unable to receive SubscriberCountStreamUpdate, the channel has been closed"
            ))?;
            let subscribers = self.subscribers.lock().await;

            let mut futures = Vec::with_capacity(subscribers.len());

            for subscriber in subscribers.iter() {
                futures.push(subscriber.send(update.clone()));
            }

            join_all(futures).await;
        }
    }
}

#[tonic::async_trait]
impl ScheduleService for MyScheduleService {
    async fn ping(
        &self,
        request: Request<event_scheduler::PingRequest>,
    ) -> Result<Response<event_scheduler::PingReply>, Status> {
        println!("Got a ping from {:?}", request.remote_addr());

        let reply = event_scheduler::PingReply {
            message: "Pong!".to_string(),
        };
        Ok(Response::new(reply))
    }

    type SubscriberCountStream = ResponseStream;

    async fn subscriber_count(
        &self,
        request: Request<event_scheduler::SubscriberCountRequest>,
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
