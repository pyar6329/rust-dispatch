use super::*;

pub trait ConnectionRpository<C: ConnectionLike> {
    fn get_conn(&self) -> C;
}

pub struct CacheRepository {
    conn: MultiplexedConnection,
}

// implements for real connection
impl ConnectionRpository<MultiplexedConnection> for CacheRepository {
    fn get_conn(&self) -> MultiplexedConnection {
        self.conn.to_owned()
    }
}

// struct for mock connection
#[cfg(test)]
pub struct MockCacheRepository {
    conn: MockRedisConnection,
}

// implements for mock connection
#[cfg(test)]
impl ConnectionRpository<MockRedisConnection> for MockCacheRepository {
    fn get_conn(&self) -> MockRedisConnection {
        self.conn.to_owned()
    }
}

pub async fn get_value<C>(conn_repo: &impl ConnectionRpository<C>, key: &str) -> Result<u64, Error>
where
    C: ConnectionLike,
{
    let mut conn = conn_repo.get_conn();
    let result: u64 = Cmd::get(key)
        .query_async(&mut conn)
        .await
        .map_err(|_| CacheError::FailedQuery)?;
    Ok(result)
}

pub async fn set_value<C>(
    conn_repo: &impl ConnectionRpository<C>,
    key: &str,
    value: &u64,
) -> Result<(), Error>
where
    C: ConnectionLike,
{
    let mut conn = conn_repo.get_conn();
    let _: () = Cmd::set_options(&key, *value, set_default_ttl())
        .query_async(&mut conn)
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

        let conn = MockRedisConnection::new(vec![MockRedisCmd::new(
            Cmd::get(key),
            Ok(Value::Int(value as i64)),
        )]);

        let cache_repo = MockCacheRepository { conn };
        let result = run_query(get_value(&cache_repo, &key)).await;

        assert!(result.is_ok());
        assert!(result.is_ok_and(|v| v == 1))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn it_return_succeed_when_set_is_called() {
        let key: &str = "foo";
        let value: u64 = 1;

        let conn = MockRedisConnection::new(vec![MockRedisCmd::new(
            Cmd::set_options(key, value, set_default_ttl()),
            Ok(Value::Okay),
        )]);

        let cache_repo = MockCacheRepository { conn };
        let result = run_query(set_value(&cache_repo, &key, &value)).await;

        assert!(result.is_ok());
    }
}
