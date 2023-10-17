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

export RUST_LOG="trace"

for month in {1..3}; do
	# Start installing opensim version.

	date="2023-$month-01"
	commit=$(./bin/osimperf-cli log --date $date --path $opensim --branch $branch)

	install="archive/opensim-$commit"
	mkdir -p $install

	export OSIMPERF_OPENSIM_SRC=$opensim
	export OSIMPERF_OPENSIM_INSTALL=$install
	export OSIMPERF_OPENSIM_BUILD="build"

	./bin/osimperf-cli install --commit $commit

	# Start running the benchmarks for each version of opensim.

	# Create a directory for collecting the results.
	results="results/opensim-$commit"
	mkdir -p "$results"

	export OSIMPERF_RESULTS=$results
	export OSIMPERF_MODELS="tests/opensim-models"

	./bin/osimperf-cli ls --tests "tests" | ./bin/osimperf-cli record \
		--iter 1

done

table_file="osimperf-results-table.md"
./bin/osimperf-cli ls --results "results" | ./bin/osimperf-cli plot --out $table_file

grip $table_file -b

# python3 csv-plot.py "results.csv"

# ./bin/osimperf-cli record --tests "tests/Arm26" --grind
