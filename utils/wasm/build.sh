#!/bin/sh

cp -r assets static
cp utils/wasm/index.html static
ls static | sed 's:^:/:' > static/index.txt
cargo web build

