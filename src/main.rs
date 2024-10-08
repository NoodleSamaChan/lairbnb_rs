use lairbnb_rs::configuration::get_configuration;
use lairbnb_rs::startup::Application;
use lairbnb_rs::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /*let subscriber = get_subscriber("lairbnb_rs".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber); */

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .compact()
        .init();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
