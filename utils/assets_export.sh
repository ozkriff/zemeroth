#!/bin/sh

# Convert one `.svg` file to many `png`s.

EXPORT_IDS="assets_src/export_ids"
INPUT_FILE="assets_src/atlas.svg"
OUT_DIR="assets/img"

mkdir -p $OUT_DIR

cat $EXPORT_IDS | tr -d '\r' | while read -r id
do
  echo Exporting "$id"
  resvg --zoom=12 --export-id="$id" $INPUT_FILE "$OUT_DIR/$id.png"
done
