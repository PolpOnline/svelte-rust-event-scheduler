mod entity_response_conversion;

use std::env;
use std::pin::Pin;
use std::sync::Arc;

use crate::grpc::event_scheduler::schedule_service_server::ScheduleServiceServer;
use crate::grpc::event_scheduler::{EventsResponse, SubscriberCountStreamUpdate};
use event_scheduler::schedule_service_server::ScheduleService;
use futures::future::join_all;
use migration::{Migrator, MigratorTrait};
use svelte_rust_event_scheduler_service::{
    sea_orm,
    sea_orm::{Database, DatabaseConnection},
    Query,
};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::codec::CompressionEncoding;
use tonic::codegen::tokio_stream;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum StartServerError {
    #[error("Failed to get address from environment variable")]
    AddressVar(#[from] env::VarError),

    #[error("Failed to parse address")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("Failed to connect to the database")]
    DatabaseConnection(#[from] sea_orm::error::DbErr),

    #[error("Failed to run migrations")]
    Migration(#[from] migration::error::Error),

    #[error("Failed to start the server")]
    Server(#[from] tonic::transport::Error),
}

pub async fn start_server() -> Result<(), StartServerError> {
    // GET the address to listen on from an environment variable
    let addr = env::var("ADDRESS")?.parse()?;

    let db_url = env::var("DATABASE_URL")?;
    let db = Database::connect(db_url).await?;
    Migrator::up(&db, None).await?;

    let schedule_service = MyScheduleService {
        database: db,
        ..Default::default()
    };

    info!("Service listening on {}", addr);

    let schedule_service_server = ScheduleServiceServer::new(schedule_service)
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd);

    Server::builder()
        .add_service(schedule_service_server)
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, thiserror::Error, tonic_thiserror::TonicThisError)]
enum ResponseStreamEventsError {
    #[error("Failed to get events")]
    #[code(Internal)]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

type ResponseStreamSubscriberCount =
    Pin<Box<dyn Stream<Item = Result<SubscriberCountStreamUpdate, Status>> + Send>>;

type ResponseStreamEvents =
    Pin<Box<dyn Stream<Item = Result<event_scheduler::EventsResponse, Status>> + Send>>;

pub mod event_scheduler {
    tonic::include_proto!("online.polp.schedule_service");
}

type SubscribersToNotify = Vec<Sender<SubscriberCountStreamUpdate>>;

#[derive(Default)]
pub struct MyScheduleService {
    subscribers: Arc<Mutex<SubscribersToNotify>>,
    database: DatabaseConnection,
}

impl MyScheduleService {
    pub async fn notify_subscribers(&self, update: SubscriberCountStreamUpdate) {
        loop {
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

    type SubscriberCountStream = ResponseStreamSubscriberCount;

    async fn subscriber_count(
        &self,
        _request: Request<event_scheduler::SubscriberCountRequest>,
    ) -> Result<Response<Self::SubscriberCountStream>, Status> {
        let (tx, rx) = mpsc::channel(4);

        self.subscribers.lock().await.push(tx);

        let output_stream = ReceiverStream::new(rx).map(Ok::<_, Status>);
        Ok(Response::new(
            Box::pin(output_stream) as Self::SubscriberCountStream
        ))
    }

    type EventsStream = ResponseStreamEvents;

    async fn events(
        &self,
        request: Request<event_scheduler::EventsRequest>,
    ) -> Result<Response<Self::EventsStream>, Status> {
        self.events_impl(&request).await.map_err(|e| e.into())
    }
}

impl MyScheduleService {
    async fn events_impl(
        &self,
        _request: &Request<event_scheduler::EventsRequest>,
    ) -> Result<Response<ResponseStreamEvents>, ResponseStreamEventsError> {
        let events = Query::get_all_events(&self.database).await?;

        let events = events.into_iter().map(|event| {
            let event: EventsResponse = event.into();
            event
        });

        let output_stream = tokio_stream::iter(events.into_iter().map(Ok::<_, Status>));

        Ok(Response::new(
            Box::pin(output_stream) as ResponseStreamEvents
        ))
    }
}
