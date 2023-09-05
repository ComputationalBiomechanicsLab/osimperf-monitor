#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

targets="osimperf-monitor osimperf-tui"
perf_home=$PWD

for target in $targets; do

	cd "osimperf/$target/package"
	OSIMPERF_HOME=$perf_home ./deploy.sh
	cd -

done
