#!/bin/sh

set -ex

cd assets && tar cf assets.tar * && cd .. && mv -f assets/assets.tar .

cargo build --target wasm32-unknown-unknown --release

rm -rf static
mkdir static
cp target/wasm32-unknown-unknown/release/zemeroth.wasm static/
cp utils/wasm/index.html static/
ls -lh static
