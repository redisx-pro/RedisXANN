#![allow(clippy::not_unsafe_ptr_arg_deref)]
mod types;

use redis_module::{redis_module, Context, RedisError, RedisResult, RedisString};

fn create_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("OK".into())
}

#[cfg(not(test))]
macro_rules! get_allocator {
    () => {
        redis_module::alloc::RedisAlloc
    };
}

#[cfg(test)]
macro_rules! get_allocator {
    () => {
        std::alloc::System
    };
}

redis_module! {
    name: "redisxann-faiss",
    version: 1,
    allocator: (get_allocator!(), get_allocator!()),
    data_types: [],
    commands: [
        ["faiss.index.create", create_index, "write", 0, 0, 0],
    ],
}
