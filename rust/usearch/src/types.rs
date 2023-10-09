use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::NonNull;
use std::{fmt, path, ptr};

use redis_module::native_types::RedisType;
use redis_module::{raw, RedisString, RedisValue};

use usearch::ffi::IndexOptions;

static INDEX_VERSION: i32 = 0;

#[derive(Default, Clone)]
pub struct IndexRedis {
    pub name: String,             // index name
    pub index_opts: IndexOptions, // usearch index options
    // pub serialization_buffer: Vec<u8>, // usearch index serialization buffer for save/load
    pub serialization_file_path: String, // usearch index serialization file path
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
            ",
            self.name,
            self.index_opts.dimensions,
            self.index_opts.metric,
            self.index_opts.quantization,
            self.index_opts.connectivity,
            self.index_opts.expansion_add,
            self.index_opts.expansion_search,
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
        mem_usage: None,
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
    let ctx = ptr::null_mut();

    let name = RedisString::create(NonNull::new(ctx), index.name);
    raw::RedisModule_SaveString.unwrap()(rdb, name.inner);

    let opts_serialized_json = serde_json::to_string(&index.index_opts).unwrap();
    let opts_cjson = CString::new(opts_serialized_json).unwrap();
    raw::save_string(rdb, opts_cjson.to_str().unwrap());

    let path_cjson = CString::new(index.serialization_file_path).unwrap();
    raw::save_string(rdb, path_cjson.to_str().unwrap());
}

unsafe extern "C" fn load_index(rdb: *mut raw::RedisModuleIO, version: i32) -> *mut c_void {
    if version != INDEX_VERSION {
        return ptr::null_mut() as *mut c_void;
    }

    let mut index = Box::new(IndexRedis::default());

    let index: *mut c_void = Box::into_raw(index) as *mut c_void;
    index
}

unsafe extern "C" fn free_index(value: *mut c_void) {
    if value.is_null() {
        // on Redis 6.0 we might get a NULL value here, so we need to handle it.
        return;
    }
    drop(Box::from_raw(value as *mut IndexRedis));
}
