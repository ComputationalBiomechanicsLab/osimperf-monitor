#!/bin/bash
set -euo pipefail

CONTEXT=$(dirname $(realpath "$0"))

echo
echo "Start BasicMillard (Opensim version = $(opensim-cmd --version))."

models="$OSPC_OPENSIM_MODELS"

if [ -z $models ]; then
	echo "ERROR: no models dir found"
	exit 1
fi

cp -r $models/Geometry .

root=$(osimperf-install-info root)
echo "$(osimperf-install-info)"
if [ -z $root ]; then
	echo "ERROR: no install found"
	exit 1
fi

build="$CONTEXT/build"
mkdir -p $build

cmake \
	-S . \
	-B $build \
	-DCMAKE_BUILD_TYPE=RelWithDebInfo \
	-DCMAKE_INSTALL_PREFIX=$CONTEXT \
	-DCMAKE_PREFIX_PATH=$root

cmake \
	--build $build \
	--target "install" \
	-j8

mkdir -p results
./bin/BasicMillard -d 0.1 -a 1e-3 -r 1e-2 -s 20 > results/plot-log
