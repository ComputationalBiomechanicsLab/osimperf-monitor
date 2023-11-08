#!/bin/bash
set -eo pipefail

source_dir=$(dirname $(dirname $(realpath "$0")))
install_dir=$PWD

target="osimperf-cli"
cargo install \
	--bin $target\
	--path "$source_dir/osimperf/$target" \
	--root "$install_dir"

cp "scripts/osimperf-default-install-opensim" "$install_dir/bin/osimperf-default-install-opensim"
cp "scripts/csv-plot.py" "$install_dir/bin/csv-plot.py"
cp "scripts/setup-benchmarks.sh" "$install_dir/bin/osimperf-setup-benchmarks"

echo "Use RUST_LOG to set log-level."
echo "Use OPENSIM_MODELS to point to opensim-models directory."

echo "dont forget to add osimperf to PATH:"
echo "PATH=$install_dir/bin:\$PATH"
