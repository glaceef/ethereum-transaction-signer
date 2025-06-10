#!/bin/sh -eu

repository_root=$(
    cd "$(dirname "$0")/.."
    pwd
)

cd "$repository_root/docker/"

docker compose run --rm -it app "$@"
