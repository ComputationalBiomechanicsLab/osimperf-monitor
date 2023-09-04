#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

PERF_HOME=$PWD
OPENSIM_CORE_SOURCE="$PERF_HOME/../../software/opensim-core-main"
INSTALL_DIR="$PERF_HOME/throwaway"

START_DATE="2023/08/01"
# Sleep for 15 minutes.
SECONDS_TO_SLEEP=900

# target="create_nodes"

# cargo install \
# 	--bin $target\
# 	--path "$PERF_HOME/osimperf/osimperf-lib" \
# 	--root "$PERF_HOME"

# ./bin/$target $@

target="compiler_service"

cargo install \
	--bin $target\
	--path "$PERF_HOME/osimperf/osimperf-lib" \
	--root "$PERF_HOME"

./bin/$target $@
