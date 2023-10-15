use serde::de::value::MapDeserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
