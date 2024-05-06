use super::*;

struct CacheConnection<C: ConnectionLike> {
    conn: C,
}

impl<C: ConnectionLike + Clone> CacheConnection<C> {
    pub async fn get_value(&self, key: &str) -> Result<u64, Error> {
        let mut conn = self.conn.to_owned();
        let result: u64 = Cmd::get(key)
            .query_async(&mut conn)
            .await
            .map_err(|_| CacheError::FailedQuery)?;
        Ok(result)
    }

    pub async fn set_value(&self, key: &str, value: &u64) -> Result<(), Error> {
        let mut conn = self.conn.to_owned();
        let _: () = Cmd::set_options(&key, *value, set_default_ttl())
            .query_async(&mut conn)
            .await
            .map_err(|_| CacheError::FailedQuery)?;
        Ok(())
    }
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

        let cache = CacheConnection { conn };
        let result = run_query(cache.get_value(&key)).await;

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

        let cache = CacheConnection { conn };
        let result = run_query(cache.set_value(&key, &value)).await;

        assert!(result.is_ok());
    }
}
