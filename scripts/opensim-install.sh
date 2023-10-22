#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Starting install script for opensim-core"

OSIMPERF_BUILD="/tmp/osimperf-opensim-build"

echo "Path to opensim-core source: $OSIMPERF_OPENSIM_SRC"
echo "Path to install: $OSIMPERF_INSTALL"
echo "Path to build: $OSIMPERF_BUILD"

mkdir -p $OSIMPERF_OPENSIM_SRC
mkdir -p $OSIMPERF_INSTALL
mkdir -p $OSIMPERF_BUILD

echo
echo "Start installing dependencies."
echo

INSTALL_DEPENDENCIES="$OSIMPERF_BUILD/dependencies"
BUILD_TYPE="RelWithDebInfo"
CASADI="OFF"
TROPTER="OFF"

cmake \
	-B "$OSIMPERF_BUILD/dependencies" \
	-S "$OSIMPERF_OPENSIM_SRC/dependencies" \
	-DCMAKE_INSTALL_PREFIX="$INSTALL_DEPENDENCIES" \
	-DCMAKE_BUILD_TYPE=$BUILD_TYPE\
	-DOPENSIM_WITH_CASADI=$CASADI\
	-DOPENSIM_WITH_TROPTER=$TROPTER

cmake \
	--build "$OSIMPERF_BUILD/dependencies" \
	-j8

echo "Completed installing dependencies."

echo
echo "Start installing opensim-core."
echo

cmake \
	-B "$OSIMPERF_BUILD/opensim-core" \
	-S $OSIMPERF_OPENSIM_SRC \
	-DCMAKE_INSTALL_PREFIX=$OSIMPERF_INSTALL \
    -DCMAKE_PREFIX_PATH=$INSTALL_DEPENDENCIES \
    -DOPENSIM_DEPENDENCIES_DIR=$INSTALL_DEPENDENCIES \
	-DCMAKE_BUILD_TYPE=$BUILD_TYPE\
	-DOPENSIM_WITH_CASADI=$CASADI\
	-DOPENSIM_WITH_TROPTER=$TROPTER \
    -DOPENSIM_BUILD_INDIVIDUAL_APPS="ON" \
    -DOPENSIM_INSTALL_UNIX_FHS="ON" \
    -DBUILD_API_EXAMPLES="ON" \
    -DBUILD_JAVA_WRAPPING="OFF" \
    -DBUILD_PYTHON_WRAPPING="OFF" \
    -DBUILD_API_ONLY="OFF" \
    -DBUILD_TESTING="OFF" \
    -DOPENSIM_DOXYGEN_USE_MATHJAX="OFF"

cmake \
	--build "$OSIMPERF_BUILD/opensim-core" \
	--target "install" \
	-j4

echo "Completed installing opensim-core."

rm -rf $OSIMPERF_BUILD
