use anyhow::Context;
use anyhow::Result;
use redis::{RedisError, Value};
use std::env;
use std::vec;
use utils::{get_redis_connection, start_redis_server_with_module};

mod utils;

#[test]
fn test_redisxann_usearch() -> Result<()> {
    // load module
    let curr_dir = env::current_dir().unwrap().to_string_lossy().to_string();
    let port: u16 = 6479;
    let _guards = vec![start_redis_server_with_module(
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
            "ip ",
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
    let eq_path = format!("{}/{}.idx", curr_dir, eq_name);
    let eq_vec = vec![
        Value::Data("name".into()),
        Value::Data(eq_name.into()),
        Value::Data("dimensions".into()),
        Value::Int(3.into()),
        Value::Data("metric".into()),
        Value::Data("IP".into()),
        Value::Data("quantization".into()),
        Value::Data("F32".into()),
        Value::Data("connectivity".into()),
        Value::Int(10.into()),
        Value::Data("expansion_add".into()),
        Value::Int(12.into()),
        Value::Data("expansion_search".into()),
        Value::Int(3.into()),
        Value::Data("serialization_file_path".into()),
        Value::Data(eq_path.into()),
        Value::Data("serialized_length".into()),
        Value::Int(112.into()),
        Value::Data("index_size".into()),
        Value::Int(0.into()),
        Value::Data("index_capacity".into()),
        Value::Int(10.into()),
        Value::Data("index_mem_usage".into()),
        Value::Int(1104.into()),
    ];
    let res: Vec<Value> = redis::cmd("usearch.index.get")
        .arg(&[test_index_name])
        .query(&mut con)
        .with_context(|| "failed to run usearch.index.get")?;
    println!("{res:?}");
    assert_eq!(res, eq_vec);

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

    // test get index node
    let test_node_name = "n1";
    let args = vec![test_index_name, test_node_name];
    let res: Vec<Value> = redis::cmd("usearch.node.get")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.node.get", file!(), line!()))?;
    let eq_node_name = format!("usearch.{}.{}", test_index_name, test_node_name);
    let eq_vec = vec![
        Value::Data("name".into()),
        Value::Data(eq_node_name.into()),
        Value::Data("data".into()),
        Value::Bulk(vec![
            Value::Data("1".into()),
            Value::Data("1".into()),
            Value::Data("1".into()),
        ]),
    ];
    println!("{res:?}");
    assert_eq!(res, eq_vec);

    // test delete index node
    let args = vec![test_index_name, test_node_name];
    let res: usize = redis::cmd("usearch.node.del")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.node.del", file!(), line!()))?;
    assert_eq!(res, 1_usize);

    // test search kann
    let mut args = vec![test_index_name, "10"];
    let q_vector = vec!["1.0"; 3];
    args.extend(q_vector);
    let res: Vec<Value> = redis::cmd("usearch.search.kann")
        .arg(&args)
        .query(&mut con)
        .with_context(|| format!("{}:{} failed to run usearch.search.kann", file!(), line!()))?;
    let eq_vec = vec![
        Value::Int(1),
        Value::Bulk(vec![
            Value::Data("similarity".into()),
            Value::Data("-2".into()),
            Value::Data("name".into()),
            Value::Data("".into()),
            Value::Data("id".into()),
            Value::Int(-1),
        ]),
    ];
    println!("{res:?}");
    assert_eq!(res, eq_vec);

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
