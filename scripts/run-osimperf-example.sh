#!/usr/bin/bash
set -e pipefail

# Example script: Paths are not portable.

env -C "../osimperf-monitor" "./scripts/install-ubuntu.sh"
export PATH="$HOME/opensim/osimperf-monitor/bin:$PATH"

export OSPC_OPENSIM_MODELS="$HOME/opensim/opensim-models"
export OSPC_OPENSIM_SRC="opensim-core" # TODO move this to /home/opensim/opensim-core

export RUST_LOG="trace"

# Build main.
git -C $OSPC_OPENSIM_SRC switch "main"
osimperf-cli install

# Build every month.
# date="2023-08-01"
# commit=$(git -C $OSPC_OPENSIM_SRC log "main" --pretty=format:%H --before=$date | head -n1)
# git -C $OSPC_OPENSIM_SRC checkout $commit
# osimperf-cli install

# Build custom branch.
name="Review"
experiment_root="$HOME/opensim/experiments/$name"

env -C $experiment_root \
osimperf-cli install \
	--name $name \
	--opensim "opensim-cbl" \
	--root "install" \
	--installer "install-opensim" \
	--force

export RUST_LOG="debug"

# Setup the benchmark tests.
for path in $(osimperf-cli ls --install $PWD/..); do
	PATH="$path/bin:$PATH" env -C "$HOME/opensim/osimperf-monitor" "./scripts/setup-benchmarks.sh"
done

# Run the benchmark tests.
for path in $(osimperf-cli ls --install $PWD/..); do
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --iter 10
done

for path in $(osimperf-cli ls --install $experiment_root); do
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --grind
done

osimperf-cli ls --results "$HOME/opensim" | osimperf-cli plot --table > osimperf-report.md
osimperf-cli ls --results "$HOME/opensim" | osimperf-cli plot > osimperf-plot.csv
csv-plot.py osimperf-plot.csv osimperf-plot.png

echo "" >> osimperf-report.md
echo "Some text here" >> osimperf-report.md
echo "" >> osimperf-report.md
echo '![Some description here](osimperf-plot.png)' >> osimperf-report.md

grip osimperf-report.md -b
