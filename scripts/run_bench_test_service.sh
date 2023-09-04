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

cd $PERF_HOME/results
rm -rf opensim-core-main-202*
cd $PERF_HOME

for i in {1..10}
do
	./bin/bench_test_service
	cat results_table.data
done
