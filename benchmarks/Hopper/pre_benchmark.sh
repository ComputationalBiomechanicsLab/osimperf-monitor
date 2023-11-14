#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start Hopper (Opensim version = $(opensim-cmd --version))."

models="$OSPC_OPENSIM_MODELS"

if [ -z $models ]; then
	echo "ERROR: no models dir found"
	exit 1
fi

cp -r $models/Geometry .

root=$(osimperf-install-info root)
if [ -z $root ]; then
	echo "ERROR: no install found"
	exit 1
fi

build="$PWD/build"
mkdir -p $build

install=$PWD

cmake \
	-S . \
	-B build \
	-DCMAKE_BUILD_TYPE=RelWithDebInfo \
	-DCMAKE_INSTALL_PREFIX=$install \
	-DCMAKE_PREFIX_PATH=$root

cmake \
	--build $build \
	--target "install" \
	-j8

$install/bin/Hopper \
	--damping "0.1" \
	--accuracy "1e-3" \
	--final-time "20" \
	--model-xml "Hopper.osim" \
	--setup-xml "setup_hopper.xml" \
	--results-dir "results"
