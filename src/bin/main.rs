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
        PROJECT_CONFIG.postgres_pool_size,
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
                &PROJECT_CONFIG.rmq_exchange,
                &PROJECT_CONFIG.rmq_exchange_type,
            ),
            Queue {
                name: &PROJECT_CONFIG.rmq_request_queue,
                routing_key: &PROJECT_CONFIG.rmq_request_queue,
            },
            on_client_message,
            "on_client_message",
        ),
        rmq_connection.start_consumer(
            Exchange::new(
                &PROJECT_CONFIG.rmq_exchange,
                &PROJECT_CONFIG.rmq_exchange_type
            ),
            Queue {
                name: &PROJECT_CONFIG.rmq_service_response_queue,
                routing_key: &PROJECT_CONFIG.rmq_service_response_queue,
            },
            on_service_message,
            "on_service_message"
        ),
        rmq_connection.start_consumer(
            Exchange::new(
                &PROJECT_CONFIG.rmq_exchange,
                &PROJECT_CONFIG.rmq_exchange_type
            ),
            Queue {
                name: &PROJECT_CONFIG.rmq_fail_table_queue,
                routing_key: &PROJECT_CONFIG.rmq_fail_table_queue,
            },
            on_fail_message,
            "on_fail_message"
        ),
        rmq_connection.start_consumer(
            Exchange::new(
                &PROJECT_CONFIG.rmq_exchange,
                &PROJECT_CONFIG.rmq_exchange_type
            ),
            Queue {
                name: &PROJECT_CONFIG.rmq_timeout_queue,
                routing_key: &PROJECT_CONFIG.rmq_timeout_queue,
            },
            on_timeout_message,
            "on_timeout_message",
        )
    );

    Ok(())
}
