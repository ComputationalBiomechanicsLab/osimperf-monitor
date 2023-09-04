#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

PERF_HOME=$PWD

target="cleanup_archive"

cargo install \
	--bin $target\
	--path "$PERF_HOME/osimperf/osimperf-lib" \
	--root "$PERF_HOME"

./bin/$target $@
