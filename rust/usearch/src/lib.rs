#![allow(clippy::not_unsafe_ptr_arg_deref)]

use redis_module::{redis_module, Context, RedisError, RedisResult, RedisString};

fn create_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("ok".into())
}

fn get_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("ok".into())
}

fn del_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("ok".into())
}

fn scan_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("ok".into())
}

redis_module! {
    name: "redisxann-usearch",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    commands: [
        ["usearch.index.create", create_index, "write", 0, 0, 0],
        ["usearch.index.get", get_index, "readonly", 0, 0, 0],
        ["usearch.index.del", del_index, "write", 0, 0, 0],
        ["usearch.index.scan", scan_index, "readonly", 0, 0, 0],
    ],
}
