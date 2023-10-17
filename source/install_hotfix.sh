#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

rm -rf $OSIMPERF_CONTEXT/build
cmake -S $OSIMPERF_MODELS/../../source -B $OSIMPERF_CONTEXT/build/ -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_INSTALL_PREFIX=$OSIMPERF_CONTEXT/install -DCMAKE_PREFIX_PATH=$OSIMPERF_OPENSIM_INSTALL/opensim-core
cmake --build $OSIMPERF_CONTEXT/build/ --target install
rm -rf $OSIMPERF_CONTEXT/build
