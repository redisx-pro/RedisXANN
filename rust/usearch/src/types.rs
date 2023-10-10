use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::{fmt, ptr};

use redis_module::native_types::RedisType;
use redis_module::{raw, RedisString, RedisValue};
use serde::{Deserialize, Serialize};

use usearch::ffi::{IndexOptions, MetricKind, ScalarKind};
use usearch::Index;

static INDEX_VERSION: i32 = 0;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
enum MKind {
    IP,
    L2sq,
    Cos,
    Pearson,
    Haversine,
    Hamming,
    Tanimoto,
    Sorensen,
}
impl MKind {
    fn map_metric_kind(&self) -> MetricKind {
        match self {
            Self::IP => MetricKind::IP,
            Self::L2sq => MetricKind::L2sq,
            Self::Cos => MetricKind::Cos,
            Self::Pearson => MetricKind::Pearson,
            Self::Haversine => MetricKind::Haversine,
            Self::Hamming => MetricKind::Hamming,
            Self::Tanimoto => MetricKind::Tanimoto,
            Self::Sorensen => MetricKind::Sorensen,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
enum SKind {
    F64,
    F32,
    F16,
    I8,
    B1,
}
impl SKind {
    fn map_scalar_kind(&self) -> ScalarKind {
        match self {
            Self::F64 => ScalarKind::F64,
            Self::F32 => ScalarKind::F32,
            Self::F16 => ScalarKind::F16,
            Self::I8 => ScalarKind::I8,
            Self::B1 => ScalarKind::B1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct IndexOpts {
    dimensions: usize,
    metric: MKind,
    quantization: SKind,
    connectivity: usize,
    expansion_add: usize,
    expansion_search: usize,
    multi: bool,
}
impl Default for IndexOpts {
    fn default() -> Self {
        Self {
            dimensions: 128,
            metric: MKind::IP,
            quantization: SKind::F32,
            connectivity: 32,
            expansion_add: 2,
            expansion_search: 3,
            multi: false,
        }
    }
}

// IndexOpts -> IndexOptions
impl From<IndexOpts> for IndexOptions {
    fn from(opts: IndexOpts) -> Self {
        IndexOptions {
            dimensions: (opts.dimensions),
            metric: (MKind::map_metric_kind(&opts.metric)),
            quantization: (SKind::map_scalar_kind(&opts.quantization)),
            connectivity: (opts.connectivity),
            expansion_add: (opts.expansion_add),
            expansion_search: (opts.expansion_search),
            multi: (opts.multi),
        }
    }
}

#[derive(Default)]
pub struct IndexRedis {
    pub name: String,          // index name
    pub index_opts: IndexOpts, // usearch index options
    pub index: Option<Index>,  // usearch index
    // pub serialization_buffer: Vec<u8>, // usearch index serialization buffer for save/load
    pub serialization_file_path: String, // usearch index serialization file path for save/load
    pub serialized_length: usize,        // usearch index saved serialized buffer length
    pub index_size: usize,               // usearch index size
    pub index_capacity: usize,           // usearch index capacity
}

impl fmt::Debug for IndexRedis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "name: {}, \
            dimensions: {}, \
            metric: {:?}, \
            quantization: {:?}, \
            connectivity: {}, \
            expansion_add: {}, \
            expansion_search: {}, \
            serialization_file_path: {}, \
            serialized_length: {}, \
            index_size: {}, \
            index_capacity: {}, \
            ",
            self.name,
            self.index_opts.dimensions,
            self.index_opts.metric,
            self.index_opts.quantization,
            self.index_opts.connectivity,
            self.index_opts.expansion_add,
            self.index_opts.expansion_search,
            self.serialization_file_path,
            self.serialized_length,
            self.index_size,
            self.index_capacity,
        )
    }
}

impl From<IndexRedis> for RedisValue {
    fn from(index: IndexRedis) -> Self {
        let mut reply: Vec<RedisValue> = Vec::new();

        reply.push("name".into());
        reply.push(index.name.as_str().into());
        reply.push("dimensions".into());
        reply.push(index.index_opts.dimensions.into());

        reply.push("metric".into());
        reply.push(format!("{:?}", index.index_opts.metric).as_str().into());
        reply.push("quantization".into());
        reply.push(
            format!("{:?}", index.index_opts.quantization)
                .as_str()
                .into(),
        );

        reply.push("connectivity".into());
        reply.push(index.index_opts.connectivity.into());
        reply.push("expansion_add".into());
        reply.push(index.index_opts.expansion_add.into());
        reply.push("expansion_search".into());
        reply.push(index.index_opts.expansion_search.into());

        reply.push("serialization_file_path".into());
        reply.push(index.serialization_file_path.as_str().into());
        reply.push("serialized_length".into());
        reply.push(index.serialized_length.into());
        reply.push("index_size".into());
        reply.push(index.index_size.into());
        reply.push("index_capacity".into());
        reply.push(index.index_capacity.into());

        reply.into()
    }
}

pub static USEARCH_INDEX_REDIS_TYPE: RedisType = RedisType::new(
    "usearch.index",
    INDEX_VERSION,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: Some(load_index),
        rdb_save: Some(save_index),
        aof_rewrite: None,
        free: Some(free_index),

        // Currently unused by Redis
        mem_usage: Some(mem_usage),
        digest: None,

        // Aux data
        aux_load: None,
        aux_save: None,
        aux_save2: None,
        aux_save_triggers: 0,

        free_effort: None,
        unlink: None,
        copy: None,
        defrag: None,

        copy2: None,
        free_effort2: None,
        mem_usage2: None,
        unlink2: None,
    },
);

unsafe extern "C" fn save_index(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    let index = Box::from_raw(value as *mut IndexRedis);

    let name_cstring = CString::new(index.name).unwrap();
    raw::save_string(rdb, name_cstring.to_str().unwrap());

    let opts_serialized_json = serde_json::to_string(&index.index_opts).unwrap();
    let opts_cjson = CString::new(opts_serialized_json).unwrap();
    raw::save_string(rdb, opts_cjson.to_str().unwrap());

    let path_cstring = CString::new(index.serialization_file_path.as_str()).unwrap();
    raw::save_string(rdb, path_cstring.to_str().unwrap());

    // match options Some/None
    if index.index.is_some() {
        let _ = index
            .index
            .unwrap()
            .save(index.serialization_file_path.as_str())
            .is_err_and(|e| {
                panic!(
                    "save usearch index to {} fail! err {}",
                    index.serialization_file_path,
                    e.to_string()
                )
            });
    } else {
        panic!("usearch index un init");
    }
}

unsafe extern "C" fn load_index(rdb: *mut raw::RedisModuleIO, encver: c_int) -> *mut c_void {
    match encver {
        0 => {
            let mut index = Box::new(IndexRedis::default());
            index.name = RedisString::from_ptr(raw::RedisModule_LoadString.unwrap()(rdb))
                .unwrap()
                .to_owned();

            let index_opts_json = RedisString::from_ptr(raw::RedisModule_LoadString.unwrap()(rdb))
                .unwrap()
                .to_owned();
            index.index_opts = serde_json::from_str(&index_opts_json).unwrap();

            index.index = Some(Index::new(&index.index_opts.clone().into()).unwrap());

            index.serialization_file_path =
                RedisString::from_ptr(raw::RedisModule_LoadString.unwrap()(rdb))
                    .unwrap()
                    .to_owned();

            let idx = index
                .index
                .as_ref()
                .unwrap_or_else(|| panic!("usearch index un init"));
            let _ = idx
                .load(index.serialization_file_path.as_str())
                .is_err_and(|e| {
                    panic!(
                        "load fail! from file {} err {}",
                        index.serialization_file_path,
                        e.to_string()
                    )
                });

            index.index_capacity = idx.capacity();
            index.index_size = idx.size();
            index.serialized_length = idx.serialized_length();

            let index: *mut c_void = Box::into_raw(index) as *mut c_void;
            index
        }
        _ => ptr::null_mut() as *mut c_void,
    }
}

unsafe extern "C" fn free_index(value: *mut c_void) {
    if value.is_null() {
        // on Redis 6.0 we might get a NULL value here, so we need to handle it.
        return;
    }
    drop(Box::from_raw(value as *mut IndexRedis));
}

unsafe extern "C" fn mem_usage(value: *const c_void) -> usize {
    let index = Box::from_raw(value as *mut IndexRedis);
    index.index.as_ref().unwrap().memory_usage()
}
