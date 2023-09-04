#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

# ==============================================================
# ======================== Config ==============================
# ==============================================================

# Folder layout.
SIMPERF_HOME="/home/pep/opensim-dev/sketches/Profiling"
OPENSIM_CORE_SOURCE="$SIMPERF_HOME/opensim-core"
ARCHIVE="$SIMPERF_HOME/archive"
SCRIPTS="$SIMPERF_HOME/scripts"
THROWAWAY="$SIMPERF_HOME/throwaway"

BUILD_DIR="$THROWAWAY/build"
INSTALL_DIR="$THROWAWAY/install"
BENCH_SOURCE_DIR="$SIMPERF_HOME/source"

while IFS= read -r line; do
	date_string=$(echo "$line" | awk -F, '{print $1}')
	commit_string=$(echo "$line" | awk -F, '{print $2}')

    # Checkout opensim core source to commit.

    # Read build config.

    # Setup build files.

    # Install.

    # Build models from source.

    # TODO: report status back.

    # Output the line
    echo "$line"
done

# End of the script
