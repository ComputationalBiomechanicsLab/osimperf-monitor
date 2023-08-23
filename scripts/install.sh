#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

PERF_HOME=$PWD
INSTALL_DIR="$PERF_HOME/throwaway"

mkdir -p $INSTALL_DIR

cargo install \
	--path "$PERF_HOME/osimperf/osimperf-monitor" \
	--root "$INSTALL_DIR"

mv "$INSTALL_DIR/bin/osimperf-monitor" "$PERF_HOME"

# Pull opensim-core
git submodule update --init --recursive
