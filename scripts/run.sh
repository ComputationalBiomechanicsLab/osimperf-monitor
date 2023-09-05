#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

targets="osimperf-monitor osimperf-tui"

for target in $targets; do
	cargo install \
		--bin $target\
		--path "osimperf/$target" \
		--root "."
done
