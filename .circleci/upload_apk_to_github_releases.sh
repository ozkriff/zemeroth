#!/bin/sh

# Based on https://gist.github.com/stefanbuck/ce788fee19ab6eb0b4447a85fc99f447

# https://github.com/settings/tokens -> circleci's env GITHUB_API_TOKEN

set -e

tag=$CIRCLE_TAG
owner=$CIRCLE_PROJECT_USERNAME
repo=$CIRCLE_PROJECT_REPONAME
filename=./zemeroth-debug.apk
GH_REPO="https://api.github.com/repos/$owner/$repo"
GH_TAGS="$GH_REPO/releases/tags/$tag"
AUTH="Authorization: token $GITHUB_API_TOKEN"

cp ./target/android-artifacts/app/build/outputs/apk/app-debug.apk $filename

# Validate token.
curl -o /dev/null -sH "$AUTH" $GH_REPO || { echo "Error: Invalid repo, token or network issue!";  exit 1; }

# Create a release
RELEASE_URL="https://api.github.com/repos/$owner/$repo/releases?access_token=$GITHUB_API_TOKEN"
curl --data "{\"tag_name\": \"$tag\"}" $RELEASE_URL

# Read asset tags
response=$(curl -sH "$AUTH" $GH_TAGS)

# Get ID of the asset based on given filename
eval $(echo "$response" | grep -m 1 "id.:" | grep -w id | tr : = | tr -cd '[[:alnum:]]=')
[ "$id" ] || { echo "Error: Failed to get release id for tag: $tag"; echo "$response\n" >&2; exit 1; }

# Upload the asset
GH_ASSET="https://uploads.github.com/repos/$owner/$repo/releases/$id/assets?name=$(basename $filename)"
curl --data-binary @"$filename" -H "$AUTH" -H "Content-Type: application/octet-stream" $GH_ASSET
