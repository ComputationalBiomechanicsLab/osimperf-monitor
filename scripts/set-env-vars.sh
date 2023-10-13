#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

export OSIMPERF_OPENSIM_SRC="$PWD/software/opensim-core"
export OSIMPERF_OPENSIM_BUILD="$PWD/build"
export OSIMPERF_OPENSIM_INSTALL="$PWD/install-latest"
export OSIMPERF_MODELS="$PWD/tests/opensim-models"
export OSIMPERF_RESULTS="$PWD/results-latest"
export PATH+=":$PWD/bin/"

mkdir -p $OSIMPERF_RESULTS
mkdir -p $OSIMPERF_OPENSIM_INSTALL

echo "OSIMPERF_OPENSIM_INSTALL=$OSIMPERF_OPENSIM_INSTALL"
echo "OSIMPERF_OPENSIM_BUILD=$OSIMPERF_OPENSIM_BUILD"
echo "OSIMPERF_OPENSIM_SRC=$OSIMPERF_OPENSIM_SRC"
echo "OSIMPERF_MODELS=$OSIMPERF_MODELS"
echo "OSIMPERF_RESULTS=$OSIMPERF_RESULTS"
