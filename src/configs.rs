use envconfig::Envconfig;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref PROJECT_CONFIG: Config =
        Config::init_from_env().expect("Failed to load envs");
}

#[allow(non_snake_case)]
#[derive(Debug, Default, Deserialize, Envconfig)]
pub struct Config {
    #[envconfig(from = "RMQ_USER", default = "user")]
    rmq_user: String,
    #[envconfig(from = "RMQ_PASSWORD", default = "bitnami")]
    rmq_password: String,
    #[envconfig(from = "RMQ_HOST", default = "rabbitmq")]
    rmq_host: String,
    #[envconfig(from = "RMQ_PORT", default = "5672")]
    rmq_port: u16,
    #[envconfig(from = "RMQ_VHOST", default = "%2f")]
    rmq_vhost: String,
    #[envconfig(from = "RMQ_PARAMS", default = "")]
    rmq_params: String,

    // service configs
    #[envconfig(from = "RMQ_REQUEST_QUEUE", default = "servicehub.q.request")]
    pub rmq_request_queue: String,
    #[envconfig(from = "RMQ_RESPONSE_QUEUE", default = "servicehub.q.response")]
    pub rmq_response_queue: String,
    #[envconfig(from = "RMQ_TIMEOUT_QUEUE", default = "timeout_requests")]
    pub rmq_timeout_queue: String,
    #[envconfig(
        from = "RMQ_SERVICE_RESPONSE_QUEUE",
        default = "servicehub.q.service_response"
    )]
    pub rmq_service_response_queue: String,
    #[envconfig(from = "RMQ_FAIL_TABLE_QUEUE", default = "servicehub.q.fail_table")]
    pub rmq_fail_table_queue: String,

    #[envconfig(from = "RMQ_DELAYED_EXCHANGE", default = "delayed_exchange")]
    pub rmq_delayed_exchange: String,
    #[envconfig(from = "RMQ_EXCHANGE", default = "servicehub")]
    pub rmq_exchange: String,
    #[envconfig(from = "RMQ_EXCHANGE_TYPE", default = "direct")]
    pub rmq_exchange_type: String,

    // Postgres configs
    #[envconfig(from = "POSTGRES_HOST", default = "postgres")]
    postgres_host: String,
    #[envconfig(from = "POSTGRES_PORT", default = "5432")]
    postgres_port: u16,
    #[envconfig(from = "POSTGRES_DB", default = "servicehub")]
    postgres_db: String,
    #[envconfig(from = "POSTGRES_PASSWORD", default = "")]
    postgres_password: String,
    #[envconfig(from = "POSTGRES_USER", default = "servicehub")]
    postgres_user: String,
    #[envconfig(from = "POSTGRES_POOL_SIZE", default = "5")]
    pub postgres_pool_size: u8,

    // Project configs
    #[envconfig(from = "AVAILABLE_SERVICES", default = "1")]
    pub available_services: String,
    #[envconfig(from = "AVAILABLE_USERS", default = "1")]
    pub available_users: String,
}

impl Config {
    pub fn get_postgres_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            self.postgres_db
        )
    }

    pub fn get_rmq_url(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}{}",
            self.rmq_user,
            self.rmq_password,
            self.rmq_host,
            self.rmq_port,
            self.rmq_vhost,
            self.rmq_params
        )
    }

    pub fn get_available_systems(&self) -> Vec<i32> {
        self.available_users
            .split(",")
            .filter(|val| !val.is_empty())
            .map(str::trim)
            .map(|val| val.parse::<i32>().unwrap())
            .collect()
    }

    pub fn get_available_services(&self) -> Vec<i32> {
        self.available_services
            .split(",")
            .filter(|val| !val.is_empty())
            .map(str::trim)
            .map(|val| val.parse::<i32>().unwrap())
            .collect()
    }
}
