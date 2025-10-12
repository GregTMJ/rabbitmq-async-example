use lapin::{Error, ErrorKind};
use log::info;
use rabbitmq_async_example::{
    configs::PROJECT_CONFIG,
    database::{check_connection, get_connection_pool},
    rmq::handlers::RmqConnectionBuilder,
    rmq::schemas::{Exchange, Queue},
    tasks::consumer::methods::{
        on_client_message, on_fail_message, on_service_message, on_timeout_message,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    env_logger::init();
    info!("All env values are set");
    info!("Checking connection");
    check_connection(&PROJECT_CONFIG.get_postgres_url())
        .await
        .unwrap();
    let pool = get_connection_pool(
        &PROJECT_CONFIG.get_postgres_url(),
        PROJECT_CONFIG.POSTGRES_POOL_SIZE,
    )
    .await
    .unwrap();
    info!("Database connection established!");
    let rmq_connection = RmqConnectionBuilder::new()
        .rmq_url(PROJECT_CONFIG.get_rmq_url())
        .sql_pool(pool)
        .build()
        .await
        .map_err(|_| Error::from(ErrorKind::NoConfiguredExecutor))?;

    let _ = tokio::join!(
        rmq_connection.start_consumer(
            Exchange::new(
                &PROJECT_CONFIG.RMQ_EXCHANGE,
                &PROJECT_CONFIG.RMQ_EXCHANGE_TYPE,
            ),
            Queue {
                name: &PROJECT_CONFIG.RMQ_REQUEST_QUEUE,
                routing_key: &PROJECT_CONFIG.RMQ_REQUEST_QUEUE,
            },
            on_client_message,
            "on_client_message",
        ),
        rmq_connection.start_consumer(
            Exchange::new(
                &PROJECT_CONFIG.RMQ_EXCHANGE,
                &PROJECT_CONFIG.RMQ_EXCHANGE_TYPE
            ),
            Queue {
                name: &PROJECT_CONFIG.RMQ_SERVICE_RESPONSE_QUEUE,
                routing_key: &PROJECT_CONFIG.RMQ_SERVICE_RESPONSE_QUEUE,
            },
            on_service_message,
            "on_service_message"
        ),
        rmq_connection.start_consumer(
            Exchange::new(
                &PROJECT_CONFIG.RMQ_EXCHANGE,
                &PROJECT_CONFIG.RMQ_EXCHANGE_TYPE
            ),
            Queue {
                name: &PROJECT_CONFIG.RMQ_FAIL_TABLE_QUEUE,
                routing_key: &PROJECT_CONFIG.RMQ_FAIL_TABLE_QUEUE,
            },
            on_fail_message,
            "on_fail_message"
        ),
        rmq_connection.start_consumer(
            Exchange::new(
                &PROJECT_CONFIG.RMQ_EXCHANGE,
                &PROJECT_CONFIG.RMQ_EXCHANGE_TYPE
            ),
            Queue {
                name: &PROJECT_CONFIG.RMQ_TIMEOUT_QUEUE,
                routing_key: &PROJECT_CONFIG.RMQ_TIMEOUT_QUEUE,
            },
            on_timeout_message,
            "on_timeout_message",
        )
    );

    Ok(())
}
