#!/bin/sh

set -ex

if [ ! -e assets.tar ]; then
  cd assets && tar cf assets.tar * && cd .. && mv assets/assets.tar .
fi

cargo build --target wasm32-unknown-unknown --release

rm -rf static
mkdir static
cp target/wasm32-unknown-unknown/release/zemeroth.wasm static/
cp utils/wasm/index.html static/
ls -lh static
