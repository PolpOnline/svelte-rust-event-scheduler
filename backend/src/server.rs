use color_eyre::Result;
fn main() -> Result<()> {
    color_eyre::install()?;

    svelte_rust_event_scheduler_api::main()
}
