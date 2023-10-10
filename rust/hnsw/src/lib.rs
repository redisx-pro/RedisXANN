#![allow(clippy::not_unsafe_ptr_arg_deref)]
mod types;

#[macro_use]
extern crate lazy_static;

use hnswcore::core::{Index, Node};
use redis_module::{
    redis_module, Context, NextArg, RedisError, RedisResult, RedisString, RedisValue,
};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use types::*;

static PREFIX: &str = "hnsw";

type IndexT = Index<f32, f32>;
type IndexArc = Arc<RwLock<IndexT>>;
lazy_static! {
    static ref INDICES: Arc<RwLock<HashMap<String, IndexArc>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

// create_index
// cmd: hnsw.index.create indexName [algo_param_key algo_param_value]
// cmd eg: hnsw.index.create idx0 dim 3 m 10 efcon 12
// return "OK" or error
fn create_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();

    if args.len() < 8 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);

    let name = format!("{}.{}", PREFIX, args.next_str()?);
    let index_name = ctx.create_string(name.clone());

    if args.next_str()?.to_lowercase() != "dim" {
        return Err(RedisError::WrongArity);
    }
    let dim = args.next_u64()? as usize;

    if args.next_str()?.to_lowercase() != "m" {
        return Err(RedisError::WrongArity);
    }
    let m = args.next_u64()? as usize;

    if args.next_str()?.to_lowercase() != "efcon" {
        return Err(RedisError::WrongArity);
    }
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
                Box::new(hnswcore::metrics::euclidean),
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

fn make_index<'a>(ctx: &'a Context, ir: &IndexRedis) -> Result<IndexT, RedisError> {
    let mut index: IndexT = ir.clone().into();

    index.nodes = HashMap::with_capacity(ir.node_count);
    for node_name in &ir.nodes {
        let key = ctx.open_key(&ctx.create_string(node_name.clone()));

        let nr = key
            .get_value::<NodeRedis>(&HNSW_NODE_REDIS_TYPE)?
            .ok_or_else(|| RedisError::String(format!("Node: {} does not exist", node_name)))?;

        let node = Node::new(node_name, &nr.data, index.m_max_0);
        index.nodes.insert(node_name.to_owned(), node);
    }

    // reconstruct nodes
    for node_name in &ir.nodes {
        let target = index.nodes.get(node_name).unwrap();

        let key = ctx.open_key(&ctx.create_string(node_name.clone()));

        let nr = key
            .get_value::<NodeRedis>(&HNSW_NODE_REDIS_TYPE)?
            .ok_or_else(|| RedisError::String(format!("Node: {} does not exist", node_name)))?;
        for layer in &nr.neighbors {
            let mut node_layer = Vec::with_capacity(layer.len());
            for neighbor in layer {
                let nn = index.nodes.get(neighbor).ok_or_else(|| {
                    RedisError::String(format!("Node: {} does not exist", neighbor))
                })?;
                node_layer.push(nn.downgrade());
            }
            target.write().neighbors.push(node_layer);
        }
    }

    // reconstruct layers
    for layer in &ir.layers {
        let mut node_layer = HashSet::with_capacity(layer.len());
        for node_name in layer {
            let node = index
                .nodes
                .get(node_name)
                .ok_or_else(|| RedisError::String(format!("Node: {} does not exist", node_name)))?;
            node_layer.insert(node.downgrade());
        }
        index.layers.push(node_layer);
    }

    // set enterpoint
    index.enterpoint = match &ir.enterpoint {
        Some(node_name) => {
            let node = index
                .nodes
                .get(node_name)
                .ok_or_else(|| RedisError::String(format!("Node: {} does not exist", node_name)))?;
            Some(node.downgrade())
        }
        None => None,
    };

    Ok(index)
}

fn load_index<'a>(ctx: &'a Context, index_name: &str) -> Result<IndexArc, RedisError> {
    let mut indices = INDICES.write().unwrap();
    // check if index is in global hashmap
    let index = match indices.entry(index_name.to_string()) {
        Entry::Occupied(o) => o.into_mut(),
        // if index isn't present, load it from redis
        Entry::Vacant(v) => {
            // get index from redis
            ctx.log_debug(format!("get key: {}", &index_name).as_str());
            let rkey = ctx.open_key(&ctx.create_string(index_name.to_string()));

            let index_redis = rkey
                .get_value::<IndexRedis>(&HNSW_INDEX_REDIS_TYPE)?
                .ok_or_else(|| {
                    RedisError::String(format!("Index: {} does not exist", index_name))
                })?;

            let index = make_index(ctx, index_redis)?;
            v.insert(Arc::new(RwLock::new(index)))
        }
    };

    Ok(index.clone())
}

// get_index
// cmd: hnsw.index.get indexName
// cmd eg: hnsw.index.get idx0
// return indexInfo or error
fn get_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();

    if args.len() != 2 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);
    let index_name = format!("{}.{}", PREFIX, args.next_str()?);

    let index = load_index(ctx, index_name.as_str())?;
    let index = index
        .try_read()
        .map_err(|e| RedisError::String(e.to_string()))?;

    ctx.log_debug(format!("Index: {:?}", index).as_str());
    ctx.log_debug(format!("Layers: {:?}", index.layers.len()).as_str());
    ctx.log_debug(format!("Nodes: {:?}", index.nodes.len()).as_str());

    let index_redis: IndexRedis = index.clone().into();

    Ok(index_redis.into())
}

fn delete_node_redis<'a>(ctx: &'a Context, node_name: &str) -> Result<(), RedisError> {
    ctx.log_debug(format!("del key: {}", node_name).as_str());
    let rkey = ctx.open_key_writable(&ctx.create_string(node_name.to_string()));
    match rkey.get_value::<NodeRedis>(&HNSW_NODE_REDIS_TYPE)? {
        Some(_) => rkey.delete()?,
        None => {
            return Err(RedisError::String(format!(
                "Node: {} does not exist",
                node_name
            )));
        }
    };

    Ok(())
}

// delete_index
// cmd: hnsw.index.del indexName
// cmd eg: hnsw.index.del idx0
// return 1 or error
fn delete_index(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();

    if args.len() != 2 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);
    let index_name = format!("{}.{}", PREFIX, args.next_str()?);

    // get index from global hashmap
    load_index(ctx, index_name.as_str())?;
    let mut indices = INDICES.write().unwrap();
    let index = indices
        .remove(&index_name)
        .ok_or_else(|| RedisError::String(format!("Index: {} does not exist", index_name)))?;
    let index = index
        .try_read()
        .map_err(|e| RedisError::String(e.to_string()))?;

    // delete index nodes from redis
    for (node_name, _) in index.nodes.iter() {
        delete_node_redis(ctx, node_name.as_str())?;
    }

    // get index from redis
    ctx.log_debug(format!("deleting index: {}", index_name).as_str());
    let rkey = ctx.open_key_writable(&ctx.create_string(index_name.clone()));

    match rkey.get_value::<IndexRedis>(&HNSW_INDEX_REDIS_TYPE)? {
        Some(_) => rkey.delete()?,
        None => {
            return Err(RedisError::String(format!(
                "Index: {} does not exist",
                index_name
            )));
        }
    };

    Ok(1_usize.into())
}

fn write_node<'a>(ctx: &'a Context, key: &str, node: NodeRedis) -> Result<(), RedisError> {
    ctx.log_debug(format!("set key: {}", key).as_str());
    let rkey = ctx.open_key_writable(&ctx.create_string(key));

    match rkey.get_value::<NodeRedis>(&HNSW_NODE_REDIS_TYPE)? {
        Some(value) => {
            value.data = node.data;
            value.neighbors = node.neighbors;
        }
        None => {
            rkey.set_value(&HNSW_NODE_REDIS_TYPE, node)?;
        }
    }
    Ok(())
}

fn update_index<'a>(ctx: &'a Context, index_name: &str, index: &IndexT) -> Result<(), RedisError> {
    let key = ctx.open_key_writable(&ctx.create_string(index_name));
    match key.get_value::<IndexRedis>(&HNSW_INDEX_REDIS_TYPE)? {
        Some(_) => {
            ctx.log_debug(format!("update index: {}", index_name).as_str());
            key.set_value::<IndexRedis>(&HNSW_INDEX_REDIS_TYPE, index.clone().into())?;
        }
        None => {
            return Err(RedisError::String(format!(
                "Index: {} does not exist",
                index_name
            )));
        }
    }
    Ok(())
}

// add_node
// cmd: hnsw.node.add indexName nodeName dataVector
// cmd eg: hnsw.node.add idx0 n1 0.6 0.1 0.1
// return "OK" or error
// todo: batch add
fn add_node(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();

    if args.len() <= 3 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);
    let index_name = format!("{}.{}", PREFIX, args.next_str()?);
    let node_name = format!("{}.{}", index_name, args.next_str()?);

    let data = args
        .into_iter()
        .map(|d| d.parse_float().unwrap() as f32)
        .collect::<Vec<f32>>();

    // load index from redisIndex
    let index = load_index(ctx, index_name.as_str())?;
    let mut index = index
        .try_write()
        .map_err(|e| RedisError::String(e.to_string()))?;

    // add node to index
    ctx.log_debug(format!("Adding node: {} to Index: {}", &node_name, &index_name).as_str());
    let up = |name: String, node: Node<f32>| {
        write_node(ctx, &name, (&node).into()).unwrap();
    };
    index
        .add_node(node_name.as_str(), &data, up)
        .map_err(|e| RedisError::String(e.error_string()))?;

    // write node to redis
    let node = index.nodes.get(&node_name).unwrap();
    write_node(ctx, node_name.as_str(), node.into())?;

    // update index in redis
    update_index(ctx, &index_name, &index)?;

    Ok("OK".into())
}

// get_node
// cmd: hnsw.node.get indexName nodeName
// cmd eg: hnsw.node.get idx0 n1
// return nodeInfo or error
// todo: batch get
fn get_node(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();

    if args.len() != 3 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);
    let index_name = format!("{}.{}", PREFIX, args.next_str()?);
    let node_name = format!("{}.{}", index_name, args.next_str()?);

    // get node from redis
    let key = ctx.open_key(&ctx.create_string(node_name.clone()));
    let value = key
        .get_value::<NodeRedis>(&HNSW_NODE_REDIS_TYPE)?
        .ok_or_else(|| RedisError::String(format!("Node: {} does not exist", &node_name)))?;

    Ok(value.into())
}

// delete_node
// cmd: hnsw.node.del indexName nodeName
// cmd eg: hnsw.node.del idx0 n1
// return 1 or error
// todo: batch del
fn delete_node(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();

    if args.len() != 3 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);
    let index_name = format!("{}.{}", PREFIX, args.next_str()?);
    let node_name = format!("{}.{}", index_name, args.next_str()?);

    // load index from redisIndex
    let index = load_index(ctx, index_name.as_str())?;
    let mut index = index
        .try_write()
        .map_err(|e| RedisError::String(e.to_string()))?;

    // get node from redisNode with Arc(atomic ref count)
    let node = index.nodes.get(&node_name).unwrap();
    if Arc::strong_count(&node.0) > 1 {
        return Err(RedisError::String(format!(
            "{} is being accessed, unable to delete. Try again later",
            node_name
        )));
    }

    // delete node from index
    let up = |name: String, node: Node<f32>| {
        write_node(ctx, &name, (&node).into()).unwrap();
    };
    index
        .delete_node(&node_name, up)
        .map_err(|e| RedisError::String(e.error_string()))?;

    // delete node from redisIndex
    delete_node_redis(ctx, &node_name)?;

    // update index in redis
    update_index(ctx, &index_name, &index)?;

    Ok(1_usize.into())
}

// search_kann
// k-Approximate Nearest Neighbors (kANN) Search
// cmd: hnsw.search.kann indexName topK queryVector
// cmd eg: hnsw.search.kann idx0 6 0.0 0.0 0.0
// return top K ANN node infos or error
// todo: add filter
fn search_kann(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    ctx.auto_memory();

    if args.len() <= 3 {
        return Err(RedisError::WrongArity);
    }

    let mut args = args.into_iter().skip(1);
    let index_name = format!("{}.{}", PREFIX, args.next_str()?);
    let k = args.next_u64()? as usize;
    let data = args
        .into_iter()
        .map(|d| d.parse_float().unwrap() as f32)
        .collect::<Vec<f32>>();

    // load index from redis
    let index = load_index(ctx, index_name.as_str())?;
    let index = index
        .try_read()
        .map_err(|e| RedisError::String(e.to_string()))?;

    ctx.log_debug(format!("Searching for {} nearest nodes in Index: {}", k, index_name).as_str());

    match index.search_kann(&data, k) {
        Ok(res) => {
            let mut reply: Vec<RedisValue> = Vec::new();
            reply.push(res.len().into());
            for r in &res {
                let sr: SearchResultRedis = r.into();
                reply.push(sr.into());
            }
            Ok(reply.into())
        }
        Err(e) => Err(RedisError::String(e.error_string())),
    }
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
    name: "redisxann-hnsw",
    version: 1,
    allocator: (get_allocator!(), get_allocator!()),
    data_types: [ HNSW_INDEX_REDIS_TYPE, HNSW_NODE_REDIS_TYPE ],
    commands: [
        [format!("{}.index.create", PREFIX), create_index, "write", 0, 0, 0],
        [format!("{}.index.get", PREFIX), get_index, "readonly", 0, 0, 0],
        [format!("{}.index.del", PREFIX), delete_index, "write", 0, 0, 0],
        [format!("{}.node.add", PREFIX), add_node, "write", 0, 0, 0],
        [format!("{}.node.get", PREFIX), get_node, "readonly", 0, 0, 0],
        [format!("{}.node.del", PREFIX), delete_node, "write", 0, 0, 0],
        [format!("{}.search.kann", PREFIX), search_kann, "readonly", 0, 0, 0],
    ],
}
