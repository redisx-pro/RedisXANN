use std::collections::HashMap;
use std::os::raw::c_int;

use redis_module::{raw::Version, RedisError};
use serde::de::value::MapDeserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct MyStruct {
    pub a: usize,
    pub b: usize,
}

#[test]
fn test_map_json() {
    let mut data: HashMap<String, usize> = HashMap::new();
    let json = serde_json::to_string(&data).unwrap();
    println!("{json:#?}");
    data.insert("a".to_string(), 1);
    println!("{data:#?}");

    let json = serde_json::to_string(&data).unwrap();
    println!("{json:#?}");
    let content = "{}";
    let mut data: HashMap<String, usize> = serde_json::from_str(content).unwrap();
    data.insert("b".to_string(), 2);
    println!("{data:#?}");
    let json = serde_json::to_string(&data).unwrap();
    println!("{json:#?}");

    let content = "{\"a\":1}";
    let mut data: HashMap<String, usize> = serde_json::from_str(content).unwrap();
    data.insert("b".to_string(), 2);
    println!("{data:#?}");
    let json = serde_json::to_string(&data).unwrap();
    println!("{json:#?}");

    let content = "{\"1\":\"a\"}";
    let mut data: HashMap<usize, String> = serde_json::from_str(content).unwrap();
    data.insert(2, "b".to_string());
    println!("{data:#?}");
    let json = serde_json::to_string(&data).unwrap();
    println!("{json:#?}");
}

#[test]
fn test_map_struct_json() {
    let content = "{\"a\":1}";
    let mut data: HashMap<&str, Value> = serde_json::from_str(content).unwrap();
    data.insert("b", Value::from(2));
    println!("{:#?}", data);
    let json = serde_json::to_string(&data).unwrap();
    println!("{:#?}", json);
    let v = MyStruct::deserialize(MapDeserializer::new(data.into_iter())).unwrap();
    println!("{:#?}", v);
    let json = serde_json::to_string(&v).unwrap();
    println!("{:#?}", json);
}

fn version_from_info(info_str: String) -> Result<Version, RedisError> {
    let regex = regex::Regex::new(
        r"(?m)\bredis_version:(?<major>[0-9]+)\.(?<minor>[0-9]+)\.(?<patch>[0-9]+)\b",
    );

    if regex.is_ok() {
        let regex = regex.unwrap();
        let mut it = regex.captures_iter(info_str.as_str());
        let caps = it.next().unwrap();
        return Ok(Version {
            major: caps["major"].parse::<c_int>().unwrap(),
            minor: caps["minor"].parse::<c_int>().unwrap(),
            patch: caps["patch"].parse::<c_int>().unwrap(),
        });
    }
    Err(RedisError::Str("Error getting redis_version"))
}

#[test]
fn test_redis_version() {
    let s = "# Server redis_version:7.2.1 redis_git_sha1:00000000 redis_git_dirty:0 redis_build_id:7b8617dd94058f85 redis_mode:standalone os:Darwin 22.6.0 x86_64 arch_bits:64 monotonic_clock:POSIX clock_gettime multiplexing_api:kqueue atomicvar_api:c11-builtin gcc_version:4.2.1 process_id:72033 process_super".to_string();

    let res = version_from_info(s);
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap(),
        Version {
            major: 7,
            minor: 2,
            patch: 1
        }
    );
}
