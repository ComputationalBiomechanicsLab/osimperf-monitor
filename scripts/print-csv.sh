#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

target="osimperf-print-csv"
cargo install \
	--bin $target\
	--path "osimperf/$target" \
	--root "."

./bin/osimperf-print-csv

python3 csv-plot.py
