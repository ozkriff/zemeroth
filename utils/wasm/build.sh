#!/bin/sh

set -ex

cargo build --target wasm32-unknown-unknown --release

rm -rf static
mkdir static
cp -r assets static/assets
cp target/wasm32-unknown-unknown/release/zemeroth.wasm static/
cp utils/wasm/index.html static/
ls -lh static
