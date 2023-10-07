use cpu_time::ProcessTime;
use faiss::index::autotune::ParameterSpace;
use faiss::selector::IdSelector;
use faiss::{index_factory, IdMap, Idx, Index, MetricType};
use hnsw_rs::prelude::*;
use rand::distributions::Uniform;
use rand::prelude::*;
use redis_module::{redis_module, Context, RedisError, RedisResult, RedisString};
use std::time::{Duration, SystemTime};
use usearch::ffi::{IndexOptions, MetricKind, ScalarKind};
use usearch::new_index;

fn get_testdata(nb_elem: usize, dim: usize) -> Vec<Vec<f32>> {
    let mut rng = thread_rng();
    let unif = Uniform::<f32>::new(0., 1.);
    let mut data = Vec::with_capacity(nb_elem);
    for _ in 0..nb_elem {
        let column = (0..dim)
            .into_iter()
            .map(|_| rng.sample(unif))
            .collect::<Vec<f32>>();
        data.push(column);
    }
    data
}

fn get_testquery(dim: usize) -> Vec<f32> {
    let mut r_vec = Vec::<f32>::with_capacity(dim);
    let mut rng = thread_rng();
    let unif = Uniform::<f32>::new(0., 1.);
    for _ in 0..dim {
        r_vec.push(rng.sample(unif));
    }
    r_vec
}

fn hnswlib_testdata(ctx: &Context) {
    //let nb_elem = 500000;
    let nb_elem = 1000;
    let dim = 25;
    // generate nb_elem colmuns vectors of dimension dim
    let data = get_testdata(nb_elem, dim);
    // give an id to each data
    let data_with_id = data.iter().zip(0..data.len()).collect();

    let ef_c = 200;
    let max_nb_connection = 15;
    let nb_layer = 16.min((nb_elem as f32).ln().trunc() as usize);
    ctx.log_notice(
        format!(
            " hnsw new max_nb_connection {}, nb_elem {}, nb_layer {}, ef_c {}",
            max_nb_connection, nb_elem, nb_layer, ef_c
        )
        .as_str(),
    );
    let hns = Hnsw::<f32, DistL2>::new(max_nb_connection, nb_elem, nb_layer, ef_c, DistL2 {});
    let mut start = ProcessTime::now();
    let mut begin_t = SystemTime::now();
    hns.parallel_insert(&data_with_id);
    let mut cpu_time: Duration = start.elapsed();
    ctx.log_notice(
        format!(
            " hnsw data count {} insertion  cpu time {:?}",
            nb_elem, cpu_time
        )
        .as_str(),
    );
    ctx.log_notice(
        format!(
            " hnsw data count {} insertion parallel,   system time {:?}",
            nb_elem,
            begin_t.elapsed().unwrap()
        )
        .as_str(),
    );
    hns.dump_layer_info();
    ctx.log_notice(
        format!(
            " parallel hnsw data nb point inserted {:?}",
            hns.get_nb_point()
        )
        .as_str(),
    );

    //
    // serial insertion
    //
    let hns = Hnsw::<f32, DistL2>::new(max_nb_connection, nb_elem, nb_layer, ef_c, DistL2 {});
    start = ProcessTime::now();
    begin_t = SystemTime::now();
    for _i in 0..data_with_id.len() {
        hns.insert(data_with_id[_i]);
    }
    cpu_time = start.elapsed();
    ctx.log_notice(format!("serial hnsw data insertion {:?}", cpu_time).as_str());
    ctx.log_notice(
        format!(
            " hnsw data insertion serial,  system time {:?}",
            begin_t.elapsed().unwrap()
        )
        .as_str(),
    );
    hns.dump_layer_info();
    ctx.log_notice(
        format!(
            " serial hnsw data nb point inserted {:?}",
            hns.get_nb_point()
        )
        .as_str(),
    );

    // top k search
    let ef_search = max_nb_connection * 2;
    let knbn = 10;
    for _iter in 0..100 {
        let r_vec = get_testquery(dim);
        let _neighbours = hns.search(&r_vec, knbn, ef_search);
        ctx.log_notice(format!(" search neighbours {:?}", _neighbours).as_str());
    }
}

fn hnswlib_test(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    if args.len() < 1 {
        return Err(RedisError::WrongArity);
    }

    hnswlib_testdata(ctx);

    ctx.log_notice(format!("{:?}", args).as_str());
    Ok("ok".into())
}

fn faiss_flat_index(_: &Context) -> RedisResult {
    let index = index_factory(4, "Flat", MetricType::L2)?;
    let mut id_index = IdMap::new(index).unwrap();
    let some_data = &[2.3_f32, 0.0, -1., 1., 1., 1., 1., 4.5, 2.3, 7.6, 1., 2.2];

    id_index.add_with_ids(some_data, &[Idx::new(4), Idx::new(8), Idx::new(12)])?;
    assert_eq!(id_index.ntotal(), 3);

    let id_sel = IdSelector::batch(&[Idx::new(4), Idx::new(12)])
        .ok()
        .unwrap();

    id_index.remove_ids(&id_sel)?;
    assert_eq!(id_index.ntotal(), 1);
    Ok("ok".into())
}

fn faiss_hnsw_index(ctx: &Context) -> RedisResult {
    // https://github.com/facebookresearch/faiss/wiki/Faiss-indexes
    // https://github.com/weedge/doraemon-nb/blob/main/faiss_composite_indexes.ipynb
    // https://github.com/facebookresearch/faiss/blob/main/faiss/index_factory.cpp
    // HNSW,Flat no train
    let index = index_factory(4, "HNSW32,Flat", MetricType::L2)?;
    let mut id_index = IdMap::new(index)?;
    // https://github.com/facebookresearch/faiss/blob/main/faiss/AutoTune.cpp
    let ps = ParameterSpace::new()?;
    ps.set_index_parameter(&mut id_index, "efConstruction", 40)?;

    let some_data = &[2.3_f32, 0.0, -1., 1., 1., 1., 1., 4.5, 2.3, 7.6, 1., 2.2];
    id_index.add_with_ids(some_data, &[Idx::new(4), Idx::new(8), Idx::new(12)])?;
    assert_eq!(id_index.ntotal(), 3);

    /*
    // remove
    let id_sel = IdSelector::batch(&[Idx::new(4), Idx::new(12)])
        .ok()
        .unwrap();
    // faiss remove_ids not implemented for this type of index(HNSW)
    id_index.remove_ids(&id_sel)?;
    assert_eq!(id_index.ntotal(), 1);
     */

    let first: [f32; 4] = [0.2, 0.1, 0.2, 0.1];
    let second: [f32; 4] = [0.2, 0.1, 0.3, 0.1];
    //assert!(index.add(&first).is_ok());
    //assert!(index.add(&second).is_ok());
    assert!(id_index.add_with_ids(&first, &[Idx::new(16)]).is_ok());
    assert!(id_index.add_with_ids(&second, &[Idx::new(20)]).is_ok());
    assert_eq!(id_index.ntotal(), 5);

    ps.set_index_parameter(&mut id_index, "efSearch", 16)
        .unwrap();
    let result = id_index.search(&first, 3)?;
    ctx.log_notice(format!("len:{}", result.labels.len()).as_str());
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

fn faiss_flat_test(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.log_notice(format!("{:?}", args).as_str());
    if args.len() < 1 {
        return Err(RedisError::WrongArity);
    }

    return faiss_flat_index(ctx);
}

fn faiss_hnsw_test(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.log_notice(format!("{:?}", args).as_str());
    if args.len() < 1 {
        return Err(RedisError::WrongArity);
    }

    return faiss_hnsw_index(ctx);
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
    assert!(index.add(44, &second).is_ok());
    assert!(index.remove(44).is_ok());

    // Read back the tags
    let results = index.search(&first, 10).unwrap();
    assert_eq!(results.keys.len(), 2);
    Ok("ok".into())
}

redis_module! {
    name: "vsstest",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    commands: [
        ["hnswlib.test", hnswlib_test, "", 0, 0, 0],
        ["faiss.flat.test", faiss_flat_test, "", 0, 0, 0],
        ["faiss.hnsw.test", faiss_hnsw_test, "", 0, 0, 0],
        ["usearch.test", usearch_test, "", 0, 0, 0],
    ],
}
