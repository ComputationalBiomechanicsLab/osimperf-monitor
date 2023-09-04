#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

targets="compiler_service terminal_ui bench_test_service"

for target in $targets; do
	cargo install \
		--bin $target\
		--path "osimperf/osimperf-lib" \
		--root "."
done
