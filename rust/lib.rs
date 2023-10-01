use faiss::{index_factory, Index, MetricType};
use redis_module::{redis_module, Context, RedisError, RedisResult, RedisString};
use usearch::ffi::{IndexOptions, MetricKind, ScalarKind};
use usearch::new_index;

fn hnswlib_test(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    if args.len() < 1 {
        return Err(RedisError::WrongArity);
    }

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("ok".into())
}

fn faiss_hnsw_test(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.log_notice(format!("{:?}", args).as_str());
    if args.len() < 1 {
        return Err(RedisError::WrongArity);
    }

    // https://github.com/facebookresearch/faiss/wiki/Faiss-indexes
    // https://github.com/weedge/doraemon-nb/blob/main/faiss_composite_indexes.ipynb
    // HNSW,Flat no train
    let mut index = index_factory(64, "HNSW,Flat", MetricType::L2)?;

    let first: [f32; 3] = [0.2, 0.1, 0.2];
    let second: [f32; 3] = [0.2, 0.1, 0.2];
    assert!(index.add(&first).is_ok());
    assert!(index.add(&second).is_ok());

    let result = index.search(&first, 2)?;
    for (i, (l, d)) in result
        .labels
        .iter()
        .zip(result.distances.iter())
        .enumerate()
    {
        ctx.log_notice(format!("#{}: {} (D={})", i + 1, *l, *d).as_str());
    }

    Ok("ok".into())
}

fn usearch_test(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.log_notice(format!("{:?}", args).as_str());
    if args.len() < 1 {
        return Err(RedisError::WrongArity);
    }

    let mut options = IndexOptions::default();
    options.dimensions = 3; // D
    options.metric = MetricKind::IP;
    options.quantization = ScalarKind::F16; // downcast
    options.connectivity = 0; // M
    options.expansion_add = 0; // ef_construction
    options.expansion_search = 0; // ef_search
    options.multi = false;

    let index = new_index(&options).unwrap();
    assert!(index.reserve(10).is_ok());
    assert!(index.capacity() >= 10);
    assert!(index.connectivity() != 0);
    assert_eq!(index.dimensions(), 3);
    assert_eq!(index.size(), 0);

    let first: [f32; 3] = [0.2, 0.1, 0.2];
    let second: [f32; 3] = [0.2, 0.1, 0.2];
    assert!(index.add(42, &first).is_ok());
    assert!(index.add(43, &second).is_ok());
    assert_eq!(index.size(), 2);

    // Read back the tags
    let results = index.search(&first, 10).unwrap();
    assert_eq!(results.keys.len(), 2);
    Ok("ok".into())
}

redis_module! {
    name: "redisxann",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    commands: [
        ["hnswlib.test", hnswlib_test, "", 0, 0, 0],
        ["faiss.hnsw.test", faiss_hnsw_test, "", 0, 0, 0],
        ["usearch.test", usearch_test, "", 0, 0, 0],
    ],
}
