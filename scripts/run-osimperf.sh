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

opensim="software/opensim-core"
branch="main"
LOG_LEVEL="debug"

for month in {1..2}; do
	date="2023-0$month-01"
	commit=$(./bin/osimperf-cli log --date $date --path $opensim --branch $branch)

	# Start installing opensim version.
	install="archive/opensim-$commit"
	mkdir -p $install

python3 csv-plot.py "results.csv"

# ./bin/osimperf-cli record --tests "tests/Arm26" --grind
