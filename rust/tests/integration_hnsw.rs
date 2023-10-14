use anyhow::Context;
use anyhow::Result;
use redis::RedisError;
use utils::{get_redis_connection, start_redis_server_with_module};

mod utils;

#[test]
fn test_redisxann_hnsw() -> Result<()> {
    let port: u16 = 6479;
    let _guards = vec![
        start_redis_server_with_module("redisxann_hnsw", port, vec![])
            .with_context(|| "failed to start redis server")?,
    ];
    let mut con =
        get_redis_connection(port).with_context(|| "failed to connect to redis server")?;

    let res: String = redis::cmd("hnsw.index.create")
        .arg(&["idx0", "dim", "3", "m", "10", "efcon", "12"])
        .query(&mut con)
        .with_context(|| "failed to run hnsw.index.create")?;
    assert_eq!(res, "OK".to_string());

    let res: Result<Vec<i32>, RedisError> =
        redis::cmd("hnsw.index.create").arg(&[""]).query(&mut con);
    if res.is_ok() {
        return Err(anyhow::Error::msg("Should return an error"));
    }

    Ok(())
}
