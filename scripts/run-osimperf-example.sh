#!/usr/bin/bash
set -e pipefail

env -C "../osimperf-monitor" "./scripts/install-ubuntu.sh"
export PATH="/home/pep/opensim/osimperf-monitor/bin:$PATH"

export OSPC_OPENSIM_MODELS="/home/pep/opensim/opensim-models"
export OSPC_OPENSIM_SRC="opensim-core" # TODO move this to /home/opensim/opensim-core

export RUST_LOG="trace"

# Build main.
git -C $OSPC_OPENSIM_SRC switch "main"
osimperf-cli install

# Build every month.
# date="2023-10-01"
# commit=$(git -C $OSPC_OPENSIM_SRC log --pretty=format:%H --before=$date | head -n1)
# git -C $OSPC_OPENSIM_SRC checkout $commit
# osimperf-cli install

# Build custom branch.
OSPC_OPENSIM_RM_BUILD_DIR="NO" \
osimperf-cli install \
	--name "wrapCyl" \
	--opensim "/home/pep/opensim/WrapCylinderMath/third-party/mod/source/opensim-core" \
	--build "$PWD/../experiments/WrapCylinderMath/build" \
	--force
	# --installer "$PWD/../experiments/WrapCylinderMath/install-opensim.sh" \

# Setup the benchmark tests.
for path in $(osimperf-cli ls --install .); do
	PATH="$path/bin:$PATH" env -C "/home/pep/opensim/osimperf-monitor" "./scripts/setup-benchmarks.sh"
done

# Run the benchmark tests.
for path in $(osimperf-cli ls --install .); do
	"osimperf-cli" ls --tests "$path" | \
		PATH="$path/bin:$PATH" \
		osimperf-cli record --iter 3
done

osimperf-cli ls --results . | osimperf-cli plot --table > osimperf-report.md
osimperf-cli ls --results . | osimperf-cli plot > osimperf-plot.csv
csv-plot.py osimperf-plot.csv osimperf-plot.png

echo "" >> osimperf-report.md
echo "Some text here" >> osimperf-report.md
echo "" >> osimperf-report.md
echo '![Some description here](osimperf-plot.png)' >> osimperf-report.md

grip osimperf-report.md -b
