use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::sync::Arc;
use std::{fmt, ptr};

use redis_module::native_types::RedisType;
use redis_module::{raw, RedisString, RedisValue};
use serde::{Deserialize, Serialize};

use usearch::ffi::{IndexOptions, MetricKind, ScalarKind};
use usearch::Index;

static INDEX_VERSION: i32 = 0;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum MKind {
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
    // From<MKind> for MetricKind
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
impl From<String> for MKind {
    fn from(opts: String) -> Self {
        match opts.as_str() {
            "f64" => Self::IP,
            "l2sq" => Self::L2sq,
            "cos" => Self::Cos,
            "pearson" => Self::Pearson,
            "haversine," => Self::Haversine,
            "hamming" => Self::Hamming,
            "tanimoto" => Self::Tanimoto,
            "sorensen" => Self::Sorensen,
            _ => Self::IP,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum SKind {
    F64,
    F32,
    F16,
    I8,
    B1,
}
impl SKind {
    // From<SKind> for ScalarKind
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
impl From<String> for SKind {
    fn from(opts: String) -> Self {
        match opts.as_str() {
            "f64" => Self::F64,
            "f32" => Self::F32,
            "f16" => Self::F16,
            "i8" => Self::I8,
            "b1" => Self::B1,
            _ => SKind::F32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct IndexOpts {
    pub dimensions: usize,
    pub metric: MKind,
    pub quantization: SKind,
    pub connectivity: usize,
    pub expansion_add: usize,
    pub expansion_search: usize,
    pub multi: bool,
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

#[derive(Default, Clone)]
pub struct IndexRedis {
    pub name: String,              // index name
    pub index_opts: IndexOpts,     // usearch index options
    pub index: Option<Arc<Index>>, // usearch index
    // pub serialization_buffer: Vec<u8>, // usearch index serialization buffer for save/load
    pub serialization_file_path: String, // usearch index serialization file path for save/load
                                         //pub serialized_length: usize,        // usearch index saved serialized buffer length
                                         //pub index_size: usize,               // usearch index size
                                         //pub index_capacity: usize,           // usearch index capacity
}

impl fmt::Debug for IndexRedis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let idx = self.index.as_ref().unwrap();
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
            idx.serialized_length(),
            idx.size(),
            idx.capacity(),
        )
    }
}

impl From<IndexRedis> for RedisValue {
    fn from(index: IndexRedis) -> Self {
        let mut reply: Vec<RedisValue> = Vec::new();

        reply.push("name".into());
        reply.push(index.name.into());
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

        let idx = index.index.unwrap();
        reply.push("serialized_length".into());
        reply.push(idx.serialized_length().into());
        reply.push("index_size".into());
        reply.push(idx.size().into());
        reply.push("idx_capacity".into());
        reply.push(idx.capacity().into());

        reply.into()
    }
}

// note: Redis requires the length of native type names to be exactly 9 characters
pub static USEARCH_INDEX_REDIS_TYPE: RedisType = RedisType::new(
    "usearchdx",
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

        copy: Some(copy_index),
        free_effort: None,
        unlink: None,
        defrag: None,

        copy2: None,
        free_effort2: None,
        mem_usage2: None,
        unlink2: None,
    },
);

unsafe extern "C" fn save_index(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    // fix: bug multi save, the second op can't get IndexRedis value.
    // let index = Box::from_raw(value as *mut IndexRedis);
    let index = unsafe { &*value.cast::<IndexRedis>() };

    let name_cstring = CString::new(index.name.as_str()).unwrap();
    raw::save_string(rdb, name_cstring.to_str().unwrap());

    let opts_serialized_json = serde_json::to_string(&index.index_opts).unwrap();
    let opts_cjson = CString::new(opts_serialized_json).unwrap();
    raw::save_string(rdb, opts_cjson.to_str().unwrap());

    let path_cstring = CString::new(index.serialization_file_path.as_str()).unwrap();
    raw::save_string(rdb, path_cstring.to_str().unwrap());

    let idx = index.index.as_ref().unwrap();
    let cap_cstring = CString::new(idx.capacity().to_string().as_str()).unwrap();
    raw::save_string(rdb, cap_cstring.to_str().unwrap());

    // match options Some/None
    if index.index.is_some() {
        let _ = idx
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

            index.index = Some(Arc::new(
                Index::new(&index.index_opts.clone().into()).unwrap(),
            ));

            index.serialization_file_path =
                RedisString::from_ptr(raw::RedisModule_LoadString.unwrap()(rdb))
                    .unwrap()
                    .to_owned();

            let cap = RedisString::from_ptr(raw::RedisModule_LoadString.unwrap()(rdb))
                .unwrap()
                .to_owned();

            let idx = index
                .index
                .as_ref()
                .unwrap_or_else(|| panic!("usearch index un init"));
            idx.load(index.serialization_file_path.as_str())
                .unwrap_or_else(|e| {
                    //panic!(
                    println!(
                        "load fail! from file {} err {}",
                        index.serialization_file_path,
                        e.to_string()
                    )
                });

            idx.reserve(cap.parse().unwrap()).unwrap_or_else(|e| {
                println!(
                    "index {} reserve cap {} err {}",
                    index.name,
                    cap,
                    e.to_string()
                )
            });

            println!("load Usearch Index {:?}", index);
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

//#[allow(unused)]
unsafe extern "C" fn copy_index(
    _: *mut raw::RedisModuleString,
    _: *mut raw::RedisModuleString,
    value: *const c_void,
) -> *mut c_void {
    let idx = unsafe { &*value.cast::<IndexRedis>() };
    let value = idx.clone();
    Box::into_raw(Box::new(value)).cast::<c_void>()
}

#[derive(Default)]
pub struct SearchResultRedis {
    pub sim: f64,
    pub name: String,
    pub id: usize,
}

impl From<SearchResultRedis> for RedisValue {
    fn from(sr: SearchResultRedis) -> Self {
        let mut reply: Vec<RedisValue> = Vec::new();

        reply.push("similarity".into());
        reply.push(sr.sim.into());

        reply.push("name".into());
        reply.push(sr.name.as_str().into());

        reply.push("id".into());
        reply.push(sr.id.into());

        reply.into()
    }
}
