use log::info;
use rabbitmq_async_example::{
    configs::PROJECT_CONFIG,
    database::check_connection,
    errors::CustomProjectErrors,
    rmq::builder::ConnectionBuilder,
    rmq::schemas::{Exchange, Queue},
    tasks::consumer::methods::{
        on_client_message, on_fail_message, on_service_message, on_timeout_message,
    },
};

#[tokio::main]
async fn main() -> Result<(), CustomProjectErrors> {
    dotenvy::dotenv().ok();
    env_logger::init();

    info!("---- All env values are set ----\n ---- Checking connection ----");

    check_connection(&PROJECT_CONFIG.get_postgres_url()).await?;

    info!("---- Database connection established! ----");

    let rmq_builder = ConnectionBuilder::new()
        .with_rmq_url(PROJECT_CONFIG.get_rmq_url())
        .with_sql_pool(
            PROJECT_CONFIG.get_postgres_url(),
            PROJECT_CONFIG.postgres_pool_size,
        )
        .build()
        .await?;

    let _ = tokio::join!(
        rmq_builder.start_consumer(
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
        rmq_builder.start_consumer(
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
        rmq_builder.start_consumer(
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
        rmq_builder.start_consumer(
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
