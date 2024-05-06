use super::{set_default_ttl, CacheError};
use anyhow::{Error, Result};
use deadpool_redis::redis::{aio::MultiplexedConnection, Cmd};

#[cfg(test)]
use redis_test::MockRedisConnection;

pub enum ConnectionType {
    Connection(MultiplexedConnection),
    #[cfg(test)]
    MockConnection(MockRedisConnection),
}

pub async fn enum_get_value(conn_type: &ConnectionType, key: &str) -> Result<u64, Error> {
    match conn_type {
        ConnectionType::Connection(connection) => {
            let mut conn = connection.to_owned();
            let result: u64 = Cmd::get(key)
                .query_async(&mut conn)
                .await
                .map_err(|_| CacheError::FailedQuery)?;
            Ok(result)
        }
        #[cfg(test)]
        ConnectionType::MockConnection(connection) => {
            let mut conn = connection.to_owned();
            let result: u64 = Cmd::get(&key)
                .query_async(&mut conn)
                .await
                .map_err(|_| CacheError::FailedQuery)?;
            Ok(result)
        }
    }
}

pub async fn enum_set_value(
    conn_type: &ConnectionType,
    key: &str,
    value: &u64,
) -> Result<(), Error> {
    match conn_type {
        ConnectionType::Connection(connection) => {
            let mut conn = connection.to_owned();
            let _: () = Cmd::set_options(&key, *value, set_default_ttl())
                .query_async(&mut conn)
                .await
                .map_err(|_| CacheError::FailedQuery)?;
            Ok(())
        }
        #[cfg(test)]
        ConnectionType::MockConnection(connection) => {
            let mut conn = connection.to_owned();
            let _: () = Cmd::set_options(&key, *value, set_default_ttl())
                .query_async(&mut conn)
                .await
                .map_err(|_| CacheError::FailedQuery)?;
            Ok(())
        }
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
        let result = run_query(enum_get_value(&ConnectionType::MockConnection(conn), &key)).await;

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
        let result = run_query(enum_set_value(
            &ConnectionType::MockConnection(conn),
            &key,
            &value,
        ))
        .await;

        assert!(result.is_ok());
    }
}
