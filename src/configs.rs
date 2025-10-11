use dotenvy::dotenv;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

lazy_static! {
    pub static ref PROJECT_CONFIG: Config = Config::from_env();
}

#[allow(non_snake_case)]
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    // RMQ Configs
    // main configs
    RMQ_USER: String,
    RMQ_PASSWORD: String,
    RMQ_HOST: String,
    RMQ_PORT: u16,
    RMQ_VHOST: String,
    RMQ_PARAMS: String,

    // service configs
    pub RMQ_REQUEST_QUEUE: String,
    pub RMQ_RESPONSE_QUEUE: String,
    pub RMQ_TIMEOUT_QUEUE: String,
    pub RMQ_DELAYED_EXCHANGE: String,
    pub RMQ_SERVICE_RESPONSE_QUEUE: String,
    pub RMQ_FAIL_TABLE_QUEUE: String,

    pub RMQ_EXCHANGE: String,
    pub RMQ_EXCHANGE_TYPE: String,

    // Postgres configs
    POSTGRES_HOST: String,
    POSTGRES_PORT: u16,
    POSTGRES_DB: String,
    POSTGRES_PASSWORD: String,
    POSTGRES_USER: String,
    pub POSTGRES_POOL_SIZE: u8,

    // Project configs
    pub AVAILABLE_SERVICES: String,
    pub AVAILABLE_USERS: String,
}

impl Config {
    pub fn get_postgres_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.POSTGRES_USER,
            self.POSTGRES_PASSWORD,
            self.POSTGRES_HOST,
            self.POSTGRES_PORT,
            self.POSTGRES_DB
        )
    }

    pub fn get_rmq_url(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}{}",
            self.RMQ_USER,
            self.RMQ_PASSWORD,
            self.RMQ_HOST,
            self.RMQ_PORT,
            self.RMQ_VHOST,
            self.RMQ_PARAMS
        )
    }

    pub fn get_available_systems(&self) -> Vec<i32> {
        self.AVAILABLE_USERS
            .split(",")
            .filter(|val| !val.is_empty())
            .map(str::trim)
            .map(|val| val.parse::<i32>().unwrap())
            .collect()
    }

    pub fn get_available_services(&self) -> Vec<i32> {
        self.AVAILABLE_SERVICES
            .split(",")
            .filter(|val| !val.is_empty())
            .map(str::trim)
            .map(|val| val.parse::<i32>().unwrap())
            .collect()
    }

    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            RMQ_USER: env::var("RMQ_USER").expect("RMQ usermust be provided"),
            RMQ_PASSWORD: env::var("RMQ_PASSWORD").expect("RMQ pass must be provided"),
            RMQ_HOST: env::var("RMQ_HOST").expect("RMQ host must be provided"),
            RMQ_PORT: env::var("RMQ_PORT")
                .unwrap_or("5672".to_owned())
                .parse::<u16>()
                .unwrap(),
            RMQ_VHOST: env::var("RMQ_VHOST").unwrap_or("%2F".to_owned()),
            RMQ_PARAMS: env::var("RMQ_PARAMS").unwrap_or_default(),

            RMQ_REQUEST_QUEUE: env::var("RMQ_REQUEST_QUEUE")
                .unwrap_or("servicehub.q.request".to_owned()),
            RMQ_RESPONSE_QUEUE: env::var("RMQ_RESPONSE_QUEUE")
                .unwrap_or("servicehub.q.response".to_owned()),
            RMQ_TIMEOUT_QUEUE: env::var("RMQ_TIMEOUT_QUEUE")
                .unwrap_or("timeout_requests".to_owned()),
            RMQ_SERVICE_RESPONSE_QUEUE: env::var("RMQ_SERVICE_RESPONSE_QUEUE")
                .unwrap_or("servicehub.q.service_response".to_owned()),
            RMQ_FAIL_TABLE_QUEUE: env::var("RMQ_FAIL_TABLE_QUEUE")
                .unwrap_or("servicehub.q.fail_table".to_owned()),

            RMQ_DELAYED_EXCHANGE: env::var("RMQ_DELAYED_EXCHANGE")
                .unwrap_or("delayed_exchange".to_owned()),
            RMQ_EXCHANGE: env::var("RMQ_EXCHANGE").unwrap(),
            RMQ_EXCHANGE_TYPE: env::var("RMQ_EXCHANGE_TYPE").unwrap_or("direct".to_owned()),

            POSTGRES_HOST: env::var("POSTGRES_HOST").expect("Database host must be provided"),
            POSTGRES_PORT: env::var("POSTGRES_PORT")
                .unwrap_or("5432".to_string())
                .parse::<u16>()
                .expect("Port must be integer"),
            POSTGRES_DB: env::var("POSTGRES_DB").unwrap_or("servicehub".to_owned()),
            POSTGRES_PASSWORD: env::var("POSTGRES_PASSWORD")
                .expect("Database password must be provided"),
            POSTGRES_USER: env::var("POSTGRES_USER").unwrap_or("servicehub".to_owned()),
            POSTGRES_POOL_SIZE: env::var("POSTGRES_POOL_SIZE")
                .unwrap_or("5".to_string())
                .parse::<u8>()
                .expect("pool size must be integer"),

            AVAILABLE_SERVICES: env::var("AVAILABLE_SERVICES").unwrap_or_default(),
            AVAILABLE_USERS: env::var("AVAILABLE_USERS").unwrap_or_default(),
        }
    }
}
