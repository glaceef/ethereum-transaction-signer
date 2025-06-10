#!/bin/sh -eu

repository_root=$(
    cd "$(dirname "$0")/.."
    pwd
)

"$repository_root/bin/_docker.sh" bash
