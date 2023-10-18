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

git -C $OSIMPERF_OPENSIM_SRC switch main

results="latest"
install="latest"
build="latest/build"

mkdir -p $install
mkdir -p $results
mkdir -p $build

export RUST_LOG="trace"

./bin/osimperf-cli install \
	--opensim "software/opensim-core" \
	--install $install \
	--build $build

./bin/osimperf-cli ls --tests "tests" | ./bin/osimperf-cli record \
	--install $install \
	--results $results \
	--models "tests/opensim-models" \
	--iter 25

./bin/osimperf-cli ls --tests "tests" | ./bin/osimperf-cli record \
	--install $install \
	--results $results \
	--models "tests/opensim-models" \
	-v -p

echo ""
./bin/osimperf-cli ls --results $results

echo ""
./bin/osimperf-cli ls --results $results | ./bin/osimperf-cli plot --out "osimperf-table.md"

grip "osimperf-table.md" -b
