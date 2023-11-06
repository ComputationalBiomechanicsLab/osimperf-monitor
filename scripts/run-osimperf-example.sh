#!/usr/bin/bash
set -ex pipefail

home="/home/developer/opensim/osimperf-collector"

# Build & install osimperf from source.
osimperf_src="$home/osimperf-monitor"
# env -C "$osimperf_src" "./scripts/install-ubuntu.sh"
export PATH="$osimperf_src/bin:$PATH"

export OSPC_OPENSIM_SRC="$home/opensim/opensim-core"
export OSPC_OPENSIM_MODELS="$home/opensim/opensim-models"

export RUST_LOG="trace"
export OSPC_OPENSIM_RUN_TESTS="OFF"
export OSPC_OPENSIM_RM_BUILD_DIR="ON"
export OSPC_NUM_JOBS="14"

# Build main.
git -C $OSPC_OPENSIM_SRC switch "main"
git -C $OSPC_OPENSIM_SRC pull
osimperf-cli install --root "install_main" --name "main"

# Run the benchmark tests.
export RUST_LOG="debug"
testRepeats=50;
for path in $(osimperf-cli ls --install .); do
	PATH="$path/bin:$PATH" env -C "$home/osimperf-monitor" "./scripts/setup-benchmarks.sh"
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --iter $testRepeats --force
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --grind
done

# Build every month.
start_date="2020-01-01";
export RUST_LOG="trace"
for (( year = $(date +%Y); year >= $(date --date=$start_date +%Y); year-- )); do
	for (( month = $(date +%m); month >= $(date --date=$start_date +%m); month-- )); do
		# Find latest opensim-core version of the month.
		date="$year-$month-01"
		commit=$(git -C $OSPC_OPENSIM_SRC log "main" --pretty=format:%H --before=$date | head -n1)
		# Checkout opensim.
		git -C $OSPC_OPENSIM_SRC checkout $commit
		# Install.
		osimperf-cli install
	done
done

# Run the benchmark tests.
export RUST_LOG="debug"
for path in $(osimperf-cli ls --install .); do
	PATH="$path/bin:$PATH" env -C "$home/osimperf-monitor" "./scripts/setup-benchmarks.sh"
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --iter $testRepeats
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --grind
done

# Build every day.
export RUST_LOG="trace"
for (( year = $(date +%Y); year >= $(date --date=$start_date +%Y); year-- )); do
	for (( month = $(date +%m); month >= $(date --date=$start_date +%m); month-- )); do
		for (( day = $(date +%d); day >= $(date --date=$start_date +%d); day-- )); do
			# Find latest opensim-core version of the month.
			date="$year-$month-$day"
			commit=$(git -C $OSPC_OPENSIM_SRC log "main" --pretty=format:%H --before=$date | head -n1)
			# Checkout opensim.
			git -C $OSPC_OPENSIM_SRC checkout $commit
			# Install.
			osimperf-cli install
		done
	done
done

# Run the benchmark tests.
export RUST_LOG="debug"
for path in $(osimperf-cli ls --install .); do
	PATH="$path/bin:$PATH" env -C "$home/osimperf-monitor" "./scripts/setup-benchmarks.sh"
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --iter $testRepeats
	osimperf-cli ls --tests "$path" | PATH="$path/bin:$PATH" osimperf-cli record --grind
done
