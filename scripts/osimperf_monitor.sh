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

START_DATE="2023/08/01"
# Sleep for 15 minutes.
SECONDS_TO_SLEEP=900

while true; do
    echo "Checking for new comits at $(date)"

    # Obtain list of last commits at each date.
    commit_history=$ARCHIVE/.commit_history.csv
    $scripts/osimperf_get_commits_since.sh.sh \
            -s $START_DATE \
            -p $OPENSIM_CORE_SOURCE \
        | $scripts/osimperf_filter_duplicate_commits.sh \
        > $commit_history
    echo "Found XXX versions of opensim-core since $start_date"

    # Check which versions still need to be compiled.
    to_be_compiled= cat $commit_history | $scripts/osimperf_filter_compiled.sh
    if (( ${#to_be_compiled[@]} -eq 0 )); then
        sleep $SECONDS_TO_SLEEP
        continue
    fi

    # Start compilation.
    echo "Setting up ${#to_be_compiled[@]} versions of opensim-core for compilation."
    echo "$to_be_compiled" | $scripts/osimperf_trigger_compilation.sh
done

    # Run tests

    # Run benchmarks

    #   Takes: path to source
    #   Produces: archive/.commit_history.csv

    # osimperf_update_compiled_versions.sh
    #   Takes:
    #       archive/.commit_history.csv
    #       source/*
    #   Executes:
    #       Get to-be-compiled list
    #       Folder setup: given (date, commit) in sandbox, and symlink to source (models)
    #       Trigger compilation of opensim core -> to install/
    #       Trigger compilation of CreateHopperBench -> to install/
    #   Produces:
    #       archive/opensim-core-DATE-HASH/install/bin/opensim-cmd
    #       archive/opensim-core-DATE-HASH/create/CreateHopperBench

    # osimperf_run_tests.sh
    #   Takes:
    #       archive/*
    #       tests/*
    #   Produces: results/opensim-core-DATE-HASH/perf.table
