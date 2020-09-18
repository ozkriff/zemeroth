#!/bin/sh

set -ex

get_stat() {
  if [ $(uname) != "Linux" ]; then
    # BSD stat
    echo stat -f "%m" $1
  else
    # GNU Coreutils stat
    echo stat -c "%Y" $1
  fi
}

if [ ! -e assets.tar ] || [ $( $(get_stat) assets.tar) -lt $( $(get_stat) assets ) ]; then
  # Check if dir "assets" has been updated
  cd assets && tar cf assets.tar * && cd .. && mv -f assets/assets.tar .
fi

cargo build --target wasm32-unknown-unknown --release

rm -rf static
mkdir static
cp target/wasm32-unknown-unknown/release/zemeroth.wasm static/
cp utils/wasm/index.html static/
ls -lh static
