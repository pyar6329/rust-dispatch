use anyhow::{anyhow, Error, Result};
use redis::{aio::MultiplexedConnection, Client, Cmd, SetExpiry, SetOptions};
use std::env;
use strum::EnumIs;
use thiserror::Error as ThisError;
use tokio::time::Duration;

const DEFAULT_TTL_5_MINUTES: u16 = 300;
const DEFAULT_TIMEOUT_30_SECONDS: u8 = 30;

pub async fn common_get_value(
    conn: &mut MultiplexedConnection,
    key: &u64,
) -> Result<String, Error> {
    let result: String = Cmd::get(&key)
        .query_async::<_, String>(conn)
        .await
        .map_err(|_| CacheError::FailedQuery)?;
    Ok(result)
}

pub async fn common_set_value(
    conn: &mut MultiplexedConnection,
    key: &u64,
    value: &str,
) -> Result<String, Error> {
    let result: String = Cmd::set_options(&key, &value, set_default_ttl())
        .query_async::<_, String>(conn)
        .await
        .map_err(|_| CacheError::FailedQuery)?;
    Ok(result)
}

#[derive(Debug, Copy, Clone, ThisError, EnumIs)]
pub enum CacheError {
    #[error("Redis URL was invalid")]
    InvalidUrl,
    #[error("Redis connection was failed")]
    ConnectionFailed,
    #[error("The query is failed")]
    FailedQuery,
}

pub async fn create_connection() -> Result<MultiplexedConnection, Error> {
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://127.0.0.1".to_string());
    let client = Client::open(redis_url).map_err(|_| anyhow!(CacheError::InvalidUrl))?;

    let response_timeout = Duration::from_secs(DEFAULT_TIMEOUT_30_SECONDS as u64);
    let connection_timeout = Duration::from_secs(DEFAULT_TIMEOUT_30_SECONDS as u64);
    client
        .get_multiplexed_async_connection_with_timeouts(response_timeout, connection_timeout)
        .await
        .map_err(|_| anyhow!(CacheError::ConnectionFailed))
}

pub fn set_default_ttl() -> SetOptions {
    SetOptions::default().with_expiration(SetExpiry::EX(DEFAULT_TTL_5_MINUTES as usize))
}
