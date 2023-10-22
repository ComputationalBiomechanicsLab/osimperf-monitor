#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

shopt -s extglob

# Inputs:
opensim="opensim-core"
branch="main"
archive="install"

mkdir -p $archive
cp "../../scripts/opensim-install.sh" "install/opensim-install.sh"
cp "../../scripts/tools-install.sh" "install/tools-install.sh"

opensim_installer="install/opensim-install.sh"
tools_installer="install/tools-install.sh"

benchmarks="run-benchmarks"
benchmarks_config="../../tests"
mkdir -p $benchmarks
cp -r $benchmarks_config/!(opensim-models) $benchmarks

export OSIMPERF_TOOLS_SRC="../../source"

# Clone opensim-core
# git clone "https://github.com/opensim-org/opensim-core.git" $opensim
# git -C $opensim submodule update --init --recursive

# Install osimperf-cli binary.
target="osimperf-cli"
cargo install \
	--bin $target\
	--path "../../osimperf/$target" \
	--root "."
export PATH="$PWD/bin:$PATH"
export RUST_LOG="trace"

for month in {8..12}; do
	# Grab opensim-core version.
	date="2023-$month-01"
	commit=$(osimperf-cli log --date $date --path $opensim --branch $branch)

	# Run installer for opensim-core.
	osimperf-cli install \
		--commit $commit \
		--opensim $opensim \
		--installer $opensim_installer

	prefix_path=$(./bin/osimperf-cli ls --install $archive --commit $commit)

	# Run installer for custom tools.
	osimperf-cli install \
		--prefix-path $prefix_path \
		--commit $commit \
		--installer $tools_installer \
		--name "tools"

	# Run all benchmarks.
	osimperf-cli ls --tests $benchmarks | osimperf-cli record \
		--prefix-path $prefix_path \
		--iter 3

	osimperf-cli ls --tests $benchmarks | osimperf-cli record \
		--prefix-path $prefix_path \
		--grind

	# Create executables for rerunning commands.
	for bench in $(osimperf-cli ls --tests $benchmarks); do
		bench_dir="$(dirname "${bench}")"
		result=$(osimperf-cli ls --results $bench_dir --commit $commit)
		result_dir="$(dirname "${result}")"

		# Create benchmark_cmd.sh for running the benchmark.
		benchmark_cmd=$(osimperf-cli record --config $bench --prefix-path $prefix_path --print)
		benchmark_cmd_file="$result_dir/benchmark_cmd.sh"
		echo "#!/bin/bash" > $benchmark_cmd_file
		echo "$benchmark_cmd" >> $benchmark_cmd_file
		chmod +x $benchmark_cmd_file

		# Create visualize_cmd.sh for running the visualizer of the benchmark.
		visualize_cmd=$(osimperf-cli record --config $bench --prefix-path $prefix_path --print --visualize)
		visualize_cmd_file="$result_dir/visualize_cmd.sh"
		echo "#!/bin/bash" > $visualize_cmd_file
		echo "$visualize_cmd" >> $visualize_cmd_file
		chmod +x $visualize_cmd_file
	done

	# List all results.
	osimperf-cli ls --results $benchmarks | osimperf-cli plot
done

table_file="osimperf-results-table.md"
osimperf-cli ls --results $benchmarks | osimperf-cli plot --out $table_file
cat $table_file
grip $table_file -b

# python3 csv-plot.py "results.csv"
