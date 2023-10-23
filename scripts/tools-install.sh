#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

# echo
# echo "Starting install script for osimperf-tools"

OSIMPERF_BUILD="/tmp/osimperf-tools-build"

echo
echo "Start installing osimperf-tools:"
echo
echo "Environmental variables:"
echo "    OSIMPERF_OPENSIM_SRC=$OSIMPERF_OPENSIM_SRC"
echo "    OSIMPERF_INSTALL=$OSIMPERF_INSTALL"
echo "    PATH=$PATH"
echo "    LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
echo
echo "    Build directory: $OSIMPERF_BUILD"
echo "    Opensim version = $(opensim-cmd --version)"

mkdir -p $OSIMPERF_OPENSIM_SRC
mkdir -p $OSIMPERF_INSTALL
mkdir -p $OSIMPERF_BUILD

cmake \
	-B "$OSIMPERF_BUILD" \
	-S "$OSIMPERF_TOOLS_SRC" \
	-DCMAKE_INSTALL_PREFIX=$OSIMPERF_INSTALL \
	-DCMAKE_BUILD_TYPE="RelWithDebInfo"

cmake \
	--build $OSIMPERF_BUILD \
	--target "install" \
	-j14

echo
echo "Removing build directory."
rm -rf $OSIMPERF_BUILD

echo "Completed installing tools."
echo
