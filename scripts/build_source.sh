#!/bin/bash
set -ueo pipefail

# Change this if this is not the root directory of project.
PERF_HOME="osimperf-monitor"

# Run from opensim-dev root directory.
if [ $(echo $(basename $PWD)) != $PERF_HOME ]; then
	echo "Please run script from $PERF_HOME directory."
	exit 1
fi

PERF_HOME=$PWD

# Build source files.

SOURCE_DIR="$PERF_HOME/source"
DEPENDENCIES_INSTALL_DIR="$PERF_HOME/archive/opensim-core"
BUILD_DIR="$PERF_HOME/build"
INSTALL_DIR="$PERF_HOME/install"
ARCHIVE_DIR="$PERF_HOME/archive"

# Configure build files.
mkdir -p $BUILD_DIR
cmake -S $SOURCE_DIR \
	-B "$BUILD_DIR" \
	-DCMAKE_EXPORT_COMPILE_COMMANDS=ON \
	-DCMAKE_PREFIX_PATH="$DEPENDENCIES_INSTALL_DIR/opensim-core-main" \
	-DCMAKE_INSTALL_PREFIX="$INSTALL_DIR"


ln -s -f $BUILD_DIR/compile_commands.json $SOURCE_DIR/compile_commands.json
echo
echo "Created symbolic link:"
echo "    $(ls -l $SOURCE_DIR/compile_commands.json)"

# Build target
cd "$BUILD_DIR"
cmake --build . --target install 2>&1 | less -FXr
cd "$PERF_HOME"

# Run it
export LD_LIBRARY_PATH="$DEPENDENCIES_INSTALL_DIR/opensim-core-main/lib:$DEPENDENCIES_INSTALL_DIR/simbody/lib"

# 	$OPENSIM_CMD "run-tool" $PERF_HOME/archive/Hopper_setup.xml

function runBinary() {
	MODEL_PATH="$ARCHIVE_DIR/Hopper-$ID.osim"
	SETUP_PATH="$ARCHIVE_DIR/Hopper-$ID-setup.xml"
	RESULTS_DIR="$ARCHIVE_DIR/Hopper-$ID-results"

		# --visualize \
	"$INSTALL_DIR/bin/Hopper" \
		--damping $DAMPING \
		--final-time $SIMTIME \
		--accuracy $ACCURACY \
		--model-xml $MODEL_PATH \
		--setup-xml $SETUP_PATH \
		--results-dir $RESULTS_DIR
		--reporter-csv >> $LOG_FILE
	echo ""
}

LOG_FILE="$ARCHIVE_DIR/plotLog.csv"

ACCURACY="1e-3"
SIMTIME="20"

DAMPING="0.01"
ID="low"
runBinary

# DAMPING="0.1"
# ID="default"
# runBinary

# DAMPING="0.5"
# ID="medium"
# runBinary

# DAMPING="1.0"
# ID="high"
# runBinary

cargo install --path "$PERF_HOME/profiler" --root "$PERF_HOME/install/"

# Run profiler
cd $PERF_HOME
"$INSTALL_DIR/bin/simperf" --path "$PERF_HOME/profiler/hopper-profiler.conf"

# echo "moving log file to /tmp/plotLog.csv"
# mv $LOG_FILE "/tmp/plotLog.csv"
# cat $LOG_FILE | ../../install/csv-plotter/bin/rust-csv-plotter
