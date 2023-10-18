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

results="latest"
install="latest"
build="latest/build"

mkdir -p $install
mkdir -p $results
mkdir -p $build

git -C $opensim switch main

export RUST_LOG="debug"

./bin/osimperf-cli install \
	--opensim $opensim \
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
