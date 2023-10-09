#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[allow(dead_code, unused_variables, unused_mut)]
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

fn get_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("".into())
}

fn del_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("".into())
}

fn scan_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("".into())
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
    name: "redisxann-usearch",
    version: 1,
    allocator: (get_allocator!(), get_allocator!()),
    data_types: [],
    commands: [
        ["usearch.index.create", create_index, "write", 0, 0, 0],
        ["usearch.index.get", get_index, "readonly", 0, 0, 0],
        ["usearch.index.del", del_index, "write", 0, 0, 0],
        ["usearch.index.scan", scan_index, "readonly", 0, 0, 0],
    ],
}
