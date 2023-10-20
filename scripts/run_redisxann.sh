#!/bin/bash

rm -rf redis && git clone https://github.com/redis/redis && \
  cd redis && \
  git checkout 7.0 && \
  make REDIS_CFLAGS='-Werror' BUILD_TLS=yes && make install && \
  which redis-server && redis-server --version

curl https://sh.rustup.rs -sSf > rustup.sh
sh rustup.sh -y

git clone https://github.com/weedge/RedisXANN.git --recursive
cd RedisXANN && source "$HOME/.cargo/env" && \
  cargo clean && \
  cargo build --lib --manifest-path rust/usearch/Cargo.toml --release

redis-server --daemonize yes \
  --loadmodule RedisXANN/target/release/libredisxann_usearch.so is_remove_serialized_file 1 \
  --port 6666 \
  --dbfilename dump.6666.rdb

ps -ef | grep redis-server | grep -v grep

