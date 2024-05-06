use super::{set_default_ttl, CacheError};
use anyhow::{Error, Result};
use deadpool_redis::redis::{
    aio::{ConnectionLike, MultiplexedConnection},
    Cmd,
};

pub async fn static_dispatch_get_value<C>(conn: &mut C, key: &str) -> Result<u64, Error>
where
    C: ConnectionLike,
{
    let result: u64 = Cmd::get(key)
        .query_async(conn)
        .await
        .map_err(|_| CacheError::FailedQuery)?;
    Ok(result)
}

pub async fn static_dispatch_set_value<C>(conn: &mut C, key: &str, value: &u64) -> Result<(), Error>
where
    C: ConnectionLike,
{
    let _: () = Cmd::set_options(&key, *value, set_default_ttl())
        .query_async(conn)
        .await
        .map_err(|_| CacheError::FailedQuery)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_query;
    use deadpool_redis::redis::Value;
    use redis_test::{MockCmd as MockRedisCmd, MockRedisConnection};

    #[tokio::test(flavor = "multi_thread")]
    async fn it_return_succeed_when_get_is_called() {
        let key: &str = "foo";
        let value: u64 = 1;

        let mut conn = MockRedisConnection::new(vec![MockRedisCmd::new(
            Cmd::get(key),
            Ok(Value::Int(value as i64)),
        )]);
        let result = run_query(static_dispatch_get_value(&mut conn, &key)).await;

        assert!(result.is_ok());
        assert!(result.is_ok_and(|v| v == 1))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn it_return_succeed_when_set_is_called() {
        let key: &str = "foo";
        let value: u64 = 1;

        let mut conn = MockRedisConnection::new(vec![MockRedisCmd::new(
            Cmd::set_options(key, value, set_default_ttl()),
            Ok(Value::Okay),
        )]);
        let result = run_query(static_dispatch_set_value(&mut conn, &key, &value)).await;

        assert!(result.is_ok());
    }
}
