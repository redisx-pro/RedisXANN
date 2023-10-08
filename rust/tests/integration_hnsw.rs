use anyhow::Context;
use anyhow::Result;
use redis::RedisError;
use utils::{get_redis_connection, start_redis_server_with_module};

mod utils;

#[test]
fn test_redisxann_hnsw() -> Result<()> {
    let port: u16 = 6479;
    let _guards = vec![start_redis_server_with_module("redisxann-hnsw", port)
        .with_context(|| "failed to start redis server")?];
    let mut con =
        get_redis_connection(port).with_context(|| "failed to connect to redis server")?;

    let res: String = redis::cmd("hnsw.index.create")
        .arg(&[3, 4])
        .query(&mut con)
        .with_context(|| "failed to run hnsw.index.create")?;
    assert_eq!(res, "Ok".to_string());

    let res: Result<Vec<i32>, RedisError> =
        redis::cmd("hnsw.index.create").arg(&[""]).query(&mut con);
    if res.is_ok() {
        return Err(anyhow::Error::msg("Should return an error"));
    }

    Ok(())
}
