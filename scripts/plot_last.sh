#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

PERF_HOME=$PWD

targets="BasicMillard BasicMillardCorrected BasicMillardDamped"

log_combined="/tmp/osimperf.csv"
echo "" > $log_combined

for target in $targets; do
	results=$(ls $PERF_HOME/results | sort | grep opensim)

	last_result=$(echo $results | awk '{print $NF}')

	res_dir="$PERF_HOME/results/$last_result/$target/output"

	log="$res_dir/stdout.log"

	ls $log

	cat $log >> $log_combined

done

# cargo install --path "$PERF_HOME/software/PlotProbe" --root "$PERF_HOME"
cat $log_combined | $PERF_HOME/bin/rust-csv-plotter
