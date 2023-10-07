#![allow(clippy::not_unsafe_ptr_arg_deref)]
mod hnsw;
mod types;

#[macro_use]
extern crate lazy_static;

use hnsw::Index;
use redis_module::{redis_module, Context, NextArg, RedisError, RedisResult, RedisString};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use types::*;

static PREFIX: &str = "hnsw";
type IndexT = Index<f32, f32>;
type IndexArc = Arc<RwLock<IndexT>>;
lazy_static! {
    static ref INDICES: Arc<RwLock<HashMap<String, IndexArc>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

fn new_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    if args.len() < 5 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);
    let name = format!("{}.{}", PREFIX, args.next_str()?);
    let index_name = ctx.create_string(name.clone());

    let dim = args.next_u64()? as usize;
    let m = args.next_u64()? as usize;
    let ef_construction = args.next_u64()? as usize;

    // write to redis
    let key = ctx.open_key_writable(&index_name);
    match key.get_value::<IndexRedis>(&HNSW_INDEX_REDIS_TYPE)? {
        Some(_) => {
            return Err(RedisError::String(format!(
                "Index: {} already exists",
                &index_name
            )));
        }
        None => {
            // create index
            let index = Index::new(
                &name,
                Box::new(hnsw::metrics::euclidean),
                dim,
                m,
                ef_construction,
            );
            ctx.log_debug(format!("{:?}", index).as_str());
            key.set_value::<IndexRedis>(&HNSW_INDEX_REDIS_TYPE, index.clone().into())?;
            // Add index to global hashmap
            INDICES
                .write()
                .unwrap()
                .insert(name, Arc::new(RwLock::new(index)));
        }
    }

    Ok("OK".into())
}

redis_module! {
    name: "redisxann-hnsw",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [
        HNSW_INDEX_REDIS_TYPE,
        HNSW_NODE_REDIS_TYPE,
    ],
    commands: [
        ["hnsw.new", new_index, "write", 0, 0, 0],
    ],
}
