#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

target="compiler_service"

cargo install \
	--bin $target\
	--path "osimperf/osimperf-lib" \
	--root "."

./bin/$target $@
