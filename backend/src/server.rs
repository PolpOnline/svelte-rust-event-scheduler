use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // Set the global default subscriber
    let filter = EnvFilter::try_new(
        "svelte_rust_event_scheduler_api=info,svelte-rust-event-scheduler-service=info",
    )
    .or_else(|_| EnvFilter::try_new("info"))
    .unwrap();

    let subscriber = fmt().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    svelte_rust_event_scheduler_api::main();
}
