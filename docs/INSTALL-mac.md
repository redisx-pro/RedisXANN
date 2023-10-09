- use faiss lib, more detail see faiss [INSTALL.md](https://github.com/Enet4/faiss/blob/c_api_head/INSTALL.md#building-from-source)
```shell
#c_api open test shared_lib avx2 openMP debug
rm -rf build && cmake -B build \
  -DFAISS_ENABLE_GPU=OFF \
  -DFAISS_ENABLE_RAFT=OFF \
  -DBUILD_TESTING=ON \
  -DFAISS_ENABLE_PYTHON=OFF \
  -DBUILD_SHARED_LIBS=ON \
  -DFAISS_ENABLE_C_API=ON \
  -DFAISS_OPT_LEVEL=avx2 \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DOpenMP_libomp_LIBRARY="/usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib" \
  -DOpenMP_CXX_FLAGS="-Xpreprocessor -fopenmp /usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib -I/usr/local/Cellar/libomp/17.0.1/include" \
  -DOpenMP_CXX_LIB_NAMES="libomp" \
  -DCMAKE_BUILD_TYPE=Debug .
make -C build -j faiss faiss_avx2
make -C build install

#c_api open test shared_lib avx2 openMP release
rm -rf build_release && cmake -B build_release \
  -DFAISS_ENABLE_GPU=OFF \
  -DFAISS_ENABLE_RAFT=OFF \
  -DBUILD_TESTING=ON \
  -DFAISS_ENABLE_PYTHON=OFF \
  -DBUILD_SHARED_LIBS=ON \
  -DFAISS_ENABLE_C_API=ON \
  -DFAISS_OPT_LEVEL=avx2 \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DOpenMP_libomp_LIBRARY="/usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib" \
  -DOpenMP_CXX_FLAGS="-Xpreprocessor -fopenmp /usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib -I/usr/local/Cellar/libomp/17.0.1/include" \
  -DOpenMP_CXX_LIB_NAMES="libomp" \
  -DCMAKE_BUILD_TYPE=Release .

make -C build_release -j faiss faiss_avx2
make -C build_release install

#c_api open test python shared_lib avx2 openMP release
rm -rf build_release && cmake -B build_release  \
  -DFAISS_ENABLE_GPU=OFF \
  -DFAISS_ENABLE_RAFT=OFF \
  -DBUILD_TESTING=ON \
  -DFAISS_ENABLE_PYTHON=ON \
  -DPython_EXECUTABLE=/usr/local/bin/python3 \
  -DSWIG_EXECUTABLE=/usr/local/bin/swig \
  -DBUILD_SHARED_LIBS=ON \
  -DFAISS_ENABLE_C_API=ON \
  -DFAISS_OPT_LEVEL=avx2 \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DOpenMP_libomp_LIBRARY="/usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib" \
  -DOpenMP_CXX_FLAGS="-Xpreprocessor -fopenmp /usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib -I/usr/local/Cellar/libomp/17.0.1/include" \
  -DOpenMP_CXX_LIB_NAMES="libomp" \
  -DCMAKE_BUILD_TYPE=Release .

make -C build_release -j faiss faiss_avx2 swigfaiss test
(cd build/faiss/python && python setup.py install)
make -C build_release install

#c_api open test python shared_lib avx2 openMP mkl release
rm -rf build_release && cmake -B build_release  \
  -DFAISS_ENABLE_GPU=OFF \
  -DFAISS_ENABLE_RAFT=OFF \
  -DBUILD_TESTING=ON \
  -DFAISS_ENABLE_PYTHON=ON \
  -DPython_EXECUTABLE=/usr/local/bin/python3 \
  -DSWIG_EXECUTABLE=/usr/local/bin/swig \
  -DBUILD_SHARED_LIBS=ON \
  -DFAISS_ENABLE_C_API=ON \
  -DBLA_VENDOR=Intel10_64lp \
  -DMKL_LIBRARIES=/opt/intel/oneapi/mkl/2023.2.0/lib \
  -DFAISS_OPT_LEVEL=avx2 \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DOpenMP_libomp_LIBRARY="/usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib" \
  -DOpenMP_CXX_FLAGS="-Xpreprocessor -fopenmp /usr/local/Cellar/libomp/17.0.1/lib/libomp.dylib -I/usr/local/Cellar/libomp/17.0.1/include" \
  -DOpenMP_CXX_LIB_NAMES="libomp" \
  -DCMAKE_BUILD_TYPE=Release .

make -C build_release -j faiss faiss_avx2 swigfaiss test
(cd build/faiss/python && python setup.py install)
make -C build_release install
```

- build redisxann modules, see [Makefile](../Makefile); run `make build`, or `make help` to build
```shell
export LIBRARY_PATH=/usr/local/lib
# http://matthew-brett.github.io/docosx/mac_runtime_link.html
# https://stackoverflow.com/questions/40602708/linking-rust-application-with-a-dynamic-library-not-in-the-runtime-linker-search
RUSTFLAGS="-C link-args=-Wl,-rpath,/usr/local/lib" cargo build
#RUSTFLAGS="-C link-args=-Wl,-rpath,/usr/local/lib" cargo build --release
```

- run examples
```shell
redis-server --loadmodule ./target/debug/examples/libvsstest.dylib --port 6370 --dbfilename dump.6370.rdb
```

- run tests, see see [Makefile](../Makefile); run `make test`, or `make help` to test 

have fun  : )
