#!/usr/bin/bash
set -ex pipefail

export OSPC_OPENSIM_SRC="$HOME/opensim/opensim-core"
export OSPC_OPENSIM_MODELS="$HOME/opensim/opensim-models"

export RUST_LOG="trace"
export OSPC_OPENSIM_RUN_TESTS="OFF"
export OSPC_OPENSIM_RM_BUILD_DIR="ON"
export OSPC_NUM_JOBS="6"

osimperf-cli ls --results "$HOME/opensim" | osimperf-cli plot --table > osimperf-report.md
osimperf-cli ls --results "$HOME/opensim" | osimperf-cli plot > osimperf-plot.csv
csv-plot.py osimperf-plot.csv osimperf-plot.png

echo "" >> osimperf-report.md
echo "Some text here" >> osimperf-report.md
echo "" >> osimperf-report.md
echo '![Some description here](osimperf-plot.png)' >> osimperf-report.md

grip osimperf-report.md -b
