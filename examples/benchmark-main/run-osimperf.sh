#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

# Inputs:
opensim="software/opensim-core" 
opensim_installer="sandbox/opensim-install.sh"
tools_installer="sandbox/tools-install.sh"
tests_dir="sandbox/opensim-install.sh sandbox/tools-install.sh"

# Install osimperf-cli binary.
target="osimperf-cli"
cargo install \
	--bin $target\
	--path "osimperf/$target" \
	--root "."
export PATH="$PWD/bin:$PATH"
export RUST_LOG="trace"

export OSIMPERF_TOOLS_SRC="../../source"

# Checkout main.
git -C $opensim switch main


osimperf-cli install \
	--opensim $opensim \
	--path "sandbox/opensim-install.sh"

for prefix_path in $(./bin/osimperf-cli ls --install "sandbox"); do
	export PATH="$prefix_path:$PATH"
	export LD_LIBRARY_PATH="$prefix_path:$LD_LIBRARY_PATH"

	osimperf-cli install \
		--opensim $opensim \
		--path "sandbox/tools-install.sh" \
		--name "tools"

	osimperf-cli ls --tests sandbox | osimperf-cli record --iter 3
done
