use anyhow::{anyhow, Error, Result};
use deadpool_redis::{
    redis::{aio::MultiplexedConnection, Client, Cmd, SetExpiry, SetOptions},
    Config, Connection, Pool, PoolConfig,
    Runtime::Tokio1,
};
use std::env;
use std::future::Future;
use strum::EnumIs;
use thiserror::Error as ThisError;
use tokio::time::{timeout, Duration};

const DEFAULT_TTL_5_MINUTES: u16 = 300;
const DEFAULT_TIMEOUT_30_SECONDS: u8 = 30;
const POOL_SIZE: u8 = 3;

pub async fn common_get_value(conn: &mut MultiplexedConnection, key: &str) -> Result<u64, Error> {
    let result: u64 = Cmd::get(&key)
        .query_async(conn)
        .await
        .map_err(|_| CacheError::FailedQuery)?;
    Ok(result)
}

pub async fn common_set_value(
    conn: &mut MultiplexedConnection,
    key: &str,
    value: &u64,
) -> Result<(), Error> {
    let _: () = Cmd::set_options(key, *value, set_default_ttl())
        .query_async(conn)
        .await
        .map_err(|_| CacheError::FailedQuery)?;
    Ok(())
}

#[derive(Debug, Copy, Clone, ThisError, EnumIs)]
pub enum CacheError {
    #[error("Redis URL was invalid")]
    InvalidUrl,
    #[error("Redis connection was failed")]
    ConnectionFailed,
    #[error("The query is failed")]
    FailedQuery,
    #[error("The query is timeout")]
    Timeout,
}

pub async fn create_pool() -> Result<Pool, Error> {
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://127.0.0.1".to_string());

    // check Redis URL is valid or invalid
    let _ = Client::open(redis_url.as_str()).map_err(|_| anyhow!(CacheError::InvalidUrl))?;

    let manager = Config::from_url(redis_url);

    // set pool size
    let manager_with_pool = Config {
        pool: Some(PoolConfig::new(POOL_SIZE as usize)),
        ..manager
    };

    // create connection pool
    manager_with_pool
        .create_pool(Some(Tokio1))
        .map_err(|_| anyhow!(CacheError::ConnectionFailed))
}

pub async fn get_conn(pool: &Pool) -> Result<Connection, Error> {
    // get connection from connection pool
    pool.get()
        .await
        .map_err(|_| anyhow!(CacheError::ConnectionFailed))
}

pub async fn run_query<F, T>(func: F) -> Result<T, Error>
where
    F: Future<Output = Result<T, Error>>,
{
    let default_err = anyhow!(CacheError::Timeout);
    let default_timeout = Duration::from_secs(DEFAULT_TIMEOUT_30_SECONDS as u64);

    timeout(default_timeout, func)
        .await
        .map_err(|_| default_err)
        .and_then(|i| i)
}

pub fn set_default_ttl() -> SetOptions {
    SetOptions::default().with_expiration(SetExpiry::EX(DEFAULT_TTL_5_MINUTES as usize))
}
