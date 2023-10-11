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

./bin/osimperf-cli install --commit "2023-09-01" --monthly

./bin/osimperf-cli record --tests "tests/Arm26" --iter 2

./bin/osimperf-cli ls --results results | ./bin/osimperf-cli plot --out "results.csv"

# ./bin/osimperf-cli record --tests "tests/Arm26" --grind
