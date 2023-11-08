#!/bin/bash
set -eo pipefail

# The following code snippet illustrates setting up all benchmarks:
#
# for path in $(osimperf-cli ls --install "$PWD"); do
# 	PATH="$path:$PATH" env -C PATH_TO_THIS_REPO ./scripts/setup-benchmarks.sh
# done
#
# You can then run all benchmarks:
#
# for path in $(osimperf-cli ls --install "$PWD"); do
# 	PATH="$path/bin:$PATH" env -C path osimperf-cli record --iter 50
# done
#
# And plot the results:
#
# osimperf-cli plot --results .

echo "Setup benchmarks for $(osimperf-install-info commit) ($(osimperf-install-info date))."

# todo move this to config.
OSIMPERF_HOME=$(dirname $(dirname $(realpath "$0")))

root_dir="$(osimperf-install-info root)"
build_dir="$root_dir/build/osimperf-tools"

mkdir -p $build_dir
mkdir -p "$root_dir/run"
mkdir -p "$root_dir/results"

cmake \
	-B "$build_dir" \
	-S "$OSIMPERF_HOME/source" \
	-DCMAKE_PREFIX_PATH="$root_dir/include" \
	-DCMAKE_INSTALL_PREFIX=$root_dir \
	-DCMAKE_BUILD_TYPE="RelWithDebInfo"

cmake \
	--build $build_dir \
	--target "install" \
	-j8

echo "Copied benchmarks scripts to $root_dir/run"
cp -r "$OSIMPERF_HOME/benchmarks" "$root_dir/run"

echo "Setup benchmarks complete."
