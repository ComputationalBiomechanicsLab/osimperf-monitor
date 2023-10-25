#!/bin/bash
set -eo pipefail

root_dir="$(osimperf-install-info path)"
build_dir="$root_dir/build"

mkdir -p $build_dir

cmake \
	-B "$build_dir" \
	-S "source" \
	-DCMAKE_PREFIX_PATH="$root_dir/include" \
	-DCMAKE_INSTALL_PREFIX=$root_dir \
	-DCMAKE_BUILD_TYPE="RelWithDebInfo"

cmake \
	--build $build_dir \
	--target "install" \
	-j8


