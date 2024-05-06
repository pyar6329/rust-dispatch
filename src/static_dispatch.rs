mod function_code;
mod method_code;
mod repository_pattern;

use super::{set_default_ttl, CacheError};
use anyhow::{Error, Result};
use deadpool_redis::redis::{
    aio::{ConnectionLike, MultiplexedConnection},
    Cmd,
};

#[cfg(test)]
use redis_test::MockRedisConnection;
