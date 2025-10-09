use lapin::Error;
use log::info;
use rabbitmq_async_example::{
    configs::Config,
    database::{check_connection, get_connection_pool},
    mapping::schemas::{Exchange, Queue},
    rmq::handlers::{RmqConnectionBuilder, start_consumer},
    tasks::consumer::methods::{on_client_message, on_service_message},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let project_configs = Config::from_env();
    env_logger::init();
    info!("All env values are set");
    info!("Checking connection");
    check_connection(&project_configs.get_postgres_url())
        .await
        .unwrap();
    let pool = get_connection_pool(
        &project_configs.get_postgres_url(),
        project_configs.POSTGRES_POOL_SIZE,
    )
    .await
    .unwrap();
    info!("Database connection established!");
    let rmq_connection = RmqConnectionBuilder::new()
        .rmq_url(project_configs.get_rmq_url())
        .sql_pool(pool)
        .build()
        .await?;

    let _ = tokio::join!(
        start_consumer(
            &rmq_connection,
            Exchange::new(
                &project_configs.RMQ_EXCHANGE,
                &project_configs.RMQ_REQUEST_QUEUE,
                &project_configs.RMQ_EXCHANGE_TYPE,
            ),
            Queue {
                name: &project_configs.RMQ_REQUEST_QUEUE
            },
            on_client_message,
            "on_client_message",
        ),
        start_consumer(
            &rmq_connection,
            Exchange::new(
                &project_configs.RMQ_EXCHANGE,
                &project_configs.RMQ_SERVICE_RESPONSE_QUEUE,
                &project_configs.RMQ_EXCHANGE_TYPE
            ),
            Queue {
                name: &project_configs.RMQ_SERVICE_RESPONSE_QUEUE
            },
            on_service_message,
            "on_service_message"
        )
    );

    Ok(())
}
