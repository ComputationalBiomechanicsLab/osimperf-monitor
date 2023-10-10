#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

target="osimperf-cli"

cargo install \
	--bin $target\
	--path "osimperf/$target" \
	--root "."

./bin/osimperf-cli install --commit "2019-11-01" --monthly

./bin/osimperf-cli record --tests "tests/Arm26" --iter 2

./bin/osimperf-cli record --tests "tests/Arm26" --grind
