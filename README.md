[![licence](https://img.shields.io/github/license/weedge/redisxann.svg)](https://github.com/weedge/redisxann/blob/main/LICENSE)

## RedisXAnn
- use rust impl redisxann modules  

## Support ANN algorithm feature
* [nmslib](https://github.com/nmslib) - [HNSW](https://github.com/nmslib/hnswlib). rust lib: [hnswlib-rs](https://github.com/jean-pierreBoth/hnswlib-rs) unsupport Dynamic update index; use [**hnswcore**](./rust/hnsw/hnswcore/) lib, index no save/load, so use redis save/load index/node, more usage_memory
* [Faiss](https://github.com/facebookresearch/faiss) HNSW with coarse quantization (eg: IndexHNSWFlat), more detail see [faiss wiki](https://github.com/facebookresearch/faiss/wiki).  rust lib: [faiss-rs](https://github.com/Enet4/faiss-rs)
* [USearch](https://github.com/unum-cloud/usearch) HNSW (**KISS**); 
  note: 
  1. if add node name, need save map (node_name<>node_id) kv, use rust hashmap, more memory; 
  2. if just add node id, id from Biz~, KISS. node meta info and node vector store separately.

## Reference
1. "Efficient and Robust approximate nearest neighbours using Hierarchical Navigable Small World Graphs" (2016,2018) [arxiv](https://arxiv.org/abs/1603.09320)
2. https://www.youtube.com/watch?v=QvKMwLjdK-s
3. https://www.youtube.com/watch?v=O98rSsuDBl8
4. https://github.com/weedge/doraemon-nb/blob/main/faiss_hnsw.ipynb
5. https://github.com/weedge/doraemon-nb/blob/main/faiss_composite_indexes.ipynb
6. https://github.com/weedge/doraemon-nb/blob/main/faiss_lsh.ipynb
7. https://weedge.github.io/post/oneday/similarity-search/1.knn-inverted-file-index/
