use anyhow::Context;
use anyhow::Ok;
use anyhow::Result;
use redis::{from_redis_value, FromRedisValue, RedisError, RedisResult, Value};
use std::collections::HashMap;
use std::env;
use std::vec;
use utils::{get_redis_connection, start_redis_server_with_module};

mod utils;

#[derive(Default, Debug)]
struct Reply {
    pub size: usize,
    pub vals: Vec<SearchResult>,
}
#[derive(Default, Debug)]
struct SearchResult {
    pub id: usize,
    pub name: String,
    pub similarity: String,
}
// https://docs.rs/redis/latest/redis/trait.FromRedisValue.html
impl FromRedisValue for SearchResult {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let mut res = SearchResult::default();
        println!("{v:?}");
        match v {
            Value::Bulk(bulk_data) if bulk_data.len() == 6 => {
                println!("{bulk_data:?}");
                for value in bulk_data.chunks(2) {
                    let field: String = from_redis_value(&value[0])?;
                    if field == "id" {
                        res.id = from_redis_value(&value[1])?;
                    }
                    if field == "name" {
                        res.name = from_redis_value(&value[1])?;
                    }
                    if field == "similarity" {
                        res.similarity = from_redis_value(&value[1])?;
                    }
                }
            }
            _ => (),
        }
        RedisResult::Ok(res)
    }
}
impl FromRedisValue for Reply {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let mut reply = Reply::default();
        match v {
            Value::Bulk(bulk_data) if bulk_data.len() > 0 => {
                reply.size = from_redis_value(&bulk_data[0])?;
                let mut vals: Vec<SearchResult> = vec![];
                for i in 1..bulk_data.len() {
                    //let val: HashMap<String, Value> = from_redis_value(&bulk_data[i])?;
                    let val: SearchResult = from_redis_value(&bulk_data[i])?;
                    vals.push(val);
                }
                reply.vals = vals;
            }
            _ => (),
        }
        RedisResult::Ok(reply)
    }
}

#[test]
fn test_redisxann_usearch() -> Result<()> {
    // load module
    let select_db = 0;
    let curr_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    let port: u16 = 6479;
    let _guards: Vec<utils::ChildGuard> = vec![start_redis_server_with_module(
        "redisxann_usearch",
        port,
        vec![
            "serialization_file_path_dir",
            curr_dir.as_str(),
            "is_remove_serialized_file",
            "ok",
        ],
    )
    .with_context(|| "failed to start redis server")?];
    let mut con =
        get_redis_connection(port).with_context(|| "failed to connect to redis server")?;

    // test create index
    let test_index_name = "test_idx0";
    let res: String = redis::cmd("usearch.index.create")
        .arg(&[
            test_index_name,
            "dim",
            "3",
            "m",
            "10",
            "efcon",
            "12",
            "metric",
            "cos",
            "quantization",
            "f32",
        ])
        .query(&mut con)
        .with_context(|| "failed to run usearch.index.create")?;
    assert_eq!(res.to_lowercase(), "ok".to_string());

    let res: Result<String, RedisError> = redis::cmd("usearch.index.create")
        .arg(&[""])
        .query(&mut con);
    if res.is_ok() {
        return Err(anyhow::Error::msg("Should return an error"));
    }

    // test get index
    let eq_name = format!("usearch.{}", test_index_name);
    let eq_path = format!("{}/{}.{}.idx", curr_dir, select_db, eq_name);
    let res: HashMap<String, Value> = redis::cmd("usearch.index.get")
        .arg(&[test_index_name])
        .query(&mut con)
        .with_context(|| "failed to run usearch.index.get")?;
    println!("{res:?}");
    assert_eq!(
        res.get("name").unwrap(),
        &Value::Data(eq_name.clone().into())
    );
    assert_eq!(res.get("dimensions").unwrap(), &Value::Int(3.into()));
    assert_eq!(res.get("metric").unwrap(), &Value::Data("Cos".into()));
    assert_eq!(res.get("quantization").unwrap(), &Value::Data("F32".into()));
    assert_eq!(res.get("connectivity").unwrap(), &Value::Int(10.into()));
    assert_eq!(res.get("expansion_add").unwrap(), &Value::Int(12.into()));
    assert_eq!(res.get("expansion_search").unwrap(), &Value::Int(3.into()));
    assert_eq!(
        res.get("serialization_file_path").unwrap(),
        &Value::Data(eq_path.into()),
    );
    assert_eq!(res.get("index_size").unwrap(), &Value::Int(0.into()));
    assert_eq!(res.get("index_capacity").unwrap(), &Value::Int(10.into()));
    assert_ne!(res.get("serialized_length").unwrap(), &Value::Int(0.into()));
    // diff mem_usage in macos/ubuntu
    assert_ne!(
        from_redis_value::<usize>(res.get("index_mem_usage").unwrap()).unwrap(),
        0
    );
    return Ok(());

    // test add index node
    let test_node_name = "n1";
    let mut args = vec![test_index_name, test_node_name];
    let test_node_vector = vec!["1.0"; 3];
    args.extend(test_node_vector);
    let res: String = redis::cmd("usearch.node.add")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.node.add", file!(), line!()))?;
    assert_eq!(res.to_lowercase(), "ok".to_string());
    let res: Result<String, RedisError> = redis::cmd("usearch.node.add").arg(&args).query(&mut con);
    if res.is_ok() {
        return Err(anyhow::Error::msg("Should return an error"));
    }

    // test get index node
    let test_node_name = "n1";
    let args = vec![test_index_name, test_node_name];
    let res: HashMap<String, Value> = redis::cmd("usearch.node.get")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.node.get", file!(), line!()))?;
    println!("{res:?}");
    let eq_node_name = format!("usearch.{}.{}", test_index_name, test_node_name);
    assert_eq!(res.get("name").unwrap(), &Value::Data(eq_node_name.into()));
    assert_eq!(
        res.get("data").unwrap(),
        &Value::Bulk(vec![
            Value::Data("1".into()),
            Value::Data("1".into()),
            Value::Data("1".into()),
        ]),
    );

    // test delete index node
    let args = vec![test_index_name, test_node_name];
    let res: usize = redis::cmd("usearch.node.del")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.node.del", file!(), line!()))?;
    assert_eq!(res, 1_usize);
    let res: Result<String, RedisError> = redis::cmd("usearch.node.get").arg(&args).query(&mut con);
    if res.is_ok() {
        return Err(anyhow::Error::msg("Should return an error"));
    }

    // test search kann
    let mut args = vec![test_index_name, "10"];
    let q_vector = vec!["1.0"; 3];
    args.extend(q_vector);
    let res: Vec<Value> = redis::cmd("usearch.search.kann")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.search.kann", file!(), line!()))?;
    println!("{res:?}");
    assert_eq!(res[0], Value::Int(0));

    // test add many index nodes to search
    for i in 0..100 {
        let tt_node_name = format!("n{}", i);
        let mut args = vec![test_index_name, tt_node_name.as_str()];
        let test_node_vector = vec!["1.0"; 3];
        args.extend(test_node_vector);
        let res: String = redis::cmd("usearch.node.add")
            .arg(&args)
            .query(&mut con)
            .with_context(|| format!("{}:{} failed to run usearch.node.add", file!(), line!()))?;
        assert_eq!(res.to_lowercase(), "ok".to_string());
    }
    // test search kann
    let k = 10;
    let binding = k.to_string();
    let mut args = vec![test_index_name, binding.as_str()];
    let q_vector = vec!["1.0"; 3];
    args.extend(q_vector);

    let res: Reply = redis::cmd("usearch.search.kann")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.search.kann", file!(), line!()))?;
    println!("{res:?}");
    assert_eq!(res.size, k);
    for val in res.vals.iter() {
        assert!(val.id > 0);
        assert!(val.name.len() > 0);
        assert!(val.similarity == "0");
    }

    // test delete index
    let res: usize = redis::cmd("usearch.index.del")
        .arg(&[test_index_name])
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.index.del", file!(), line!()))?;
    assert_eq!(res, 1_usize);

    let res: Result<String, RedisError> = redis::cmd("usearch.index.del")
        .arg(&[test_index_name])
        .query(&mut con);
    if res.is_ok() {
        return Err(anyhow::Error::msg("Should return an error"));
    }

    Ok(())
}
