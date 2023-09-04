#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

PERF_HOME=$PWD
OPENSIM_CORE_SOURCE="$PERF_HOME/../../software/opensim-core-main"
INSTALL_DIR="$PERF_HOME/throwaway"

# cargo install \
# 	--bin $target\
# 	--path "$PERF_HOME/osimperf/osimperf-lib" \
# 	--root "$PERF_HOME"

# ./bin/$target $@

target="bench_test_service"

cargo install \
	--bin $target\
	--path "$PERF_HOME/osimperf/osimperf-lib" \
	--root "$PERF_HOME"

./bin/$target $@
