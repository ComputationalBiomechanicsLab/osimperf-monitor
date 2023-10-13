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

for month in {1..12}; do
	date="2023-0$month-01"
	commit=$(./bin/osimperf-cli log --date $date --path $opensim --branch $branch)

	# Start installing opensim version.
	install="archive/opensim-$commit"
	mkdir -p $install

	RUST_LOG=$LOG_LEVEL ./bin/osimperf-cli install \
		--opensim $opensim \
		--install $install \
		--build "build" \
		--commit $commit

	# Start running the benchmarks for each version of opensim.

	# Create a directory for collecting the results.
	results="results/opensim-$commit"
	mkdir -p "$results"

	./bin/osimperf-cli ls --tests "tests/Arm26" | RUST_LOG=$LOG_LEVEL ./bin/osimperf-cli record \
		--install $install \
		--results $results \
		--models "tests/opensim-models" \
		--iter 10 \
		--grind

done

# python3 csv-plot.py "results.csv"

# ./bin/osimperf-cli record --tests "tests/Arm26" --grind
