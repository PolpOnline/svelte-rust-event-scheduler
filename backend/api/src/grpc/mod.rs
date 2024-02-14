mod entity_response_conversion;

use axum::Router;
use std::env;
use std::pin::Pin;
use std::sync::Arc;

use crate::grpc::event_scheduler::schedule_service_server::ScheduleServiceServer;
use crate::grpc::event_scheduler::{
    EventUsersStatusRequest, EventUsersStatusResponse, EventsResponse, SubscriberCountStreamUpdate,
};
use event_scheduler::schedule_service_server::ScheduleService;
use futures::future::join_all;
use migration::{Migrator, MigratorTrait};
use svelte_rust_event_scheduler_service::{
    sea_orm,
    sea_orm::{Database, DatabaseConnection},
    Mutation, Query,
};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::metadata::MetadataValue;
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
    Axum(#[from] axum::Error),

    #[error("Failed to start the server")]
    Io(#[from] std::io::Error),

    #[error("Failed to start the server")]
    Hyper(#[from] hyper::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub async fn start_server() -> Result<(), StartServerError> {
    // GET the address to listen on from an environment variable
    let addr = env::var("ADDRESS")
        .unwrap_or("[::]:80".to_string())
        .parse()?;

    let db_url = env::var("DATABASE_URL")?;
    let db = Database::connect(db_url).await?;
    Migrator::up(&db, None).await?;

    let schedule_service = MyScheduleService {
        database: db,
        ..Default::default()
    };

    let schedule_service_server =
        ScheduleServiceServer::with_interceptor(schedule_service, check_auth_interceptor);

    let schedule_service_server = tonic_web::enable(schedule_service_server);
    let app = Router::new().route(
        "/online.polp.schedule_service.ScheduleService/*rpc",
        axum::routing::any_service(schedule_service_server.clone()),
    );

    info!("Service will listen on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Uses openid connect to authenticate the user
fn check_auth_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    match request.metadata().get("authorization") {
        Some(t) if token == t => Ok(request),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}

#[derive(Debug, thiserror::Error, tonic_thiserror::TonicThisError)]
enum ResponseStreamEventsError {
    #[error("Failed to get events")]
    #[code(Internal)]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

#[derive(Debug, thiserror::Error, tonic_thiserror::TonicThisError)]
enum EventSubscriptionError {
    #[error("Failed to subscribe to event")]
    #[code(Internal)]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

#[derive(Debug, thiserror::Error, tonic_thiserror::TonicThisError)]
enum EventJoinResponseError {
    #[error("Failed to join event")]
    #[code(Internal)]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

#[derive(Debug, thiserror::Error, tonic_thiserror::TonicThisError)]
enum EventLeaveResponseError {
    #[error("Failed to leave event")]
    #[code(Internal)]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

#[derive(Debug, thiserror::Error, tonic_thiserror::TonicThisError)]
enum ResponseStreamEventUsersStatusError {
    #[error("Failed to get event users status")]
    #[code(Internal)]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

type ResponseStreamSubscriberCount =
    Pin<Box<dyn Stream<Item = Result<SubscriberCountStreamUpdate, Status>> + Send>>;

type ResponseStreamEvents = Pin<Box<dyn Stream<Item = Result<EventsResponse, Status>> + Send>>;

type ResponseStreamEventUsersStatus =
    Pin<Box<dyn Stream<Item = Result<EventUsersStatusResponse, Status>> + Send>>;

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
        let subscribers = self.subscribers.lock().await;

        let mut futures = Vec::with_capacity(subscribers.len());

        for subscriber in subscribers.iter() {
            futures.push(subscriber.send(update.clone()));
        }

        join_all(futures).await;
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
        self.events_impl(request).await.map_err(|e| e.into())
    }

    async fn subscribe_to_events(
        &self,
        request: Request<event_scheduler::EventSubscriptionRequest>,
    ) -> Result<Response<event_scheduler::EventSubscriptionResponse>, Status> {
        self.subscribe_to_events_impl(request)
            .await
            .map_err(|e| e.into())
    }

    async fn join_event(
        &self,
        request: Request<event_scheduler::EventJoinRequest>,
    ) -> Result<Response<event_scheduler::EventJoinResponse>, Status> {
        self.join_event_impl(request).await.map_err(|e| e.into())
    }

    async fn leave_event(
        &self,
        request: Request<event_scheduler::EventLeaveRequest>,
    ) -> Result<Response<event_scheduler::EventLeaveResponse>, Status> {
        self.leave_event_impl(request).await.map_err(|e| e.into())
    }

    type EventUsersStatusStream = ResponseStreamEventUsersStatus;

    async fn event_users_status(
        &self,
        request: Request<EventUsersStatusRequest>,
    ) -> Result<Response<Self::EventUsersStatusStream>, Status> {
        self.event_users_status_impl(request)
            .await
            .map_err(|e| e.into())
    }
}

impl MyScheduleService {
    async fn events_impl(
        &self,
        _request: Request<event_scheduler::EventsRequest>,
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

    async fn subscribe_to_events_impl(
        &self,
        request: Request<event_scheduler::EventSubscriptionRequest>,
    ) -> Result<Response<event_scheduler::EventSubscriptionResponse>, EventSubscriptionError> {
        let body = request.into_inner();

        info!(
            "User {} subscribed to events {:?}",
            body.user_id, body.event_ids
        );

        Mutation::subscribe_to_events(&self.database, body.user_id, &body.event_ids).await?;

        let counts = Query::get_events_user_count_by_ids(&self.database, body.event_ids).await?;

        for count in counts {
            self.notify_subscribers(SubscriberCountStreamUpdate {
                id: count.event_id,
                subscriber_count: count.count,
            })
            .await;
        }

        Ok(Response::new(event_scheduler::EventSubscriptionResponse {}))
    }

    async fn join_event_impl(
        &self,
        request: Request<event_scheduler::EventJoinRequest>,
    ) -> Result<Response<event_scheduler::EventJoinResponse>, EventJoinResponseError> {
        let body = request.into_inner();

        Mutation::join_event(&self.database, body.user_id, body.event_id).await?;

        Ok(Response::new(event_scheduler::EventJoinResponse {}))
    }

    async fn leave_event_impl(
        &self,
        request: Request<event_scheduler::EventLeaveRequest>,
    ) -> Result<Response<event_scheduler::EventLeaveResponse>, EventLeaveResponseError> {
        let body = request.into_inner();

        Mutation::leave_event(&self.database, body.user_id, body.event_id).await?;

        Ok(Response::new(event_scheduler::EventLeaveResponse {}))
    }

    async fn event_users_status_impl(
        &self,
        request: Request<EventUsersStatusRequest>,
    ) -> Result<Response<ResponseStreamEventUsersStatus>, ResponseStreamEventUsersStatusError> {
        let body = request.into_inner();

        let event_users =
            Query::event_users_status(&self.database, body.event_id, body.round).await?;

        let output_stream = tokio_stream::iter(
            event_users
                .into_iter()
                .map(|e| e.into())
                .map(Ok::<_, Status>),
        );

        Ok(Response::new(
            Box::pin(output_stream) as ResponseStreamEventUsersStatus
        ))
    }
}
