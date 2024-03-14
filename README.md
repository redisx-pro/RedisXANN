[![licence](https://img.shields.io/github/license/weedge/redisxann.svg)](https://github.com/weedge/redisxann/blob/main/LICENSE)

## RedisXAnn
- use rust impl redisxann modules  

## Support ANN algorithm feature

- HNSW
  * [nmslib](https://github.com/nmslib) - [HNSW](https://github.com/nmslib/hnswlib). rust lib: [hnswlib-rs](https://github.com/jean-pierreBoth/hnswlib-rs) unsupport Dynamic update index; use [**hnswcore**](./rust/hnsw/hnswcore/) lib, index no save/load, so use redis save/load index/node, more usage_memory
  * [Faiss](https://github.com/facebookresearch/faiss) HNSW with coarse quantization (eg: IndexHNSWFlat), more detail see [faiss wiki](https://github.com/facebookresearch/faiss/wiki).  rust lib: [faiss-rs](https://github.com/Enet4/faiss-rs)
  * [USearch](https://github.com/unum-cloud/usearch) HNSW (**KISS**); 
    note: 
    1. if add node name, need save map (node_name<>node_id) kv, use rust hashmap, more memory; 
    2. if just add node id, id from Biz~, KISS. node meta info and node vector store separately.

## CMD
- create index: `usearch.index.create indexName [algo_param_key algo_param_value]`
- get index: `usearch.index.get indexName`
- del index: `usearch.index.del indexName`
- add node: `usearch.node.add indexName nodeName nodeVector`
- add vector: `usearch.node.add_id indexName vectorId dataVector`
- get node: `usearch.node.get indexName nodeName`
- get vector: `usearch.node.get_id indexName vectorId`
- del node: `usearch.node.del indexName nodeName`
- del vector: `usearch.node.del_id indexName vectorId`
- kann search: `usearch.search.kann indexName topK queryVector [filter_string] [param_key param_value]`

## SDK
- python: [redisx-py](https://github.com/weedge/redisx-py)

## Cases
1. [**implement cross modal image text retrieval**](https://github.com/weedge/doraemon-nb/blob/main/redisxann_usearch_implement_cross_modal_image_text_retrieval.ipynb)
2. [**implement approximate query for molecular geometries**](https://github.com/weedge/doraemon-nb/blob/main/redisxann_usearch_implement_approximate_query_for_molecular_geometries.ipynb)
3. [**implement Geo Spatial Indexing**](https://github.com/weedge/doraemon-nb/blob/main/redisxann_usearch_implement_Geo_Spatial_Indexing.ipynb)

## Reference
1. "Efficient and Robust approximate nearest neighbours using Hierarchical Navigable Small World Graphs" (2016,2018) [arxiv](https://arxiv.org/abs/1603.09320)
2. https://www.youtube.com/watch?v=QvKMwLjdK-s
3. https://www.youtube.com/watch?v=O98rSsuDBl8
4. https://github.com/weedge/doraemon-nb/blob/main/faiss_hnsw.ipynb
5. https://github.com/weedge/doraemon-nb/blob/main/faiss_composite_indexes.ipynb
6. https://github.com/weedge/doraemon-nb/blob/main/faiss_lsh.ipynb
7. https://weedge.github.io/post/oneday/similarity-search/1.knn-inverted-file-index/
8. https://github.com/unum-cloud/usearch/blob/main/docs/benchmarks.md
