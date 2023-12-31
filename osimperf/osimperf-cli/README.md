# OpenSim Performance Collector

This project contains CLI tool for benchmarking performance of opensim-core.

### Installing

Use `osimperf-cli install` subcommand for installing a version of opensim, e.g.:

`osimperf-cli install --opensim software/opensim-core --installer install-main/install-opensim.sh`

will install currently checked out version at `software/opensim-core` using the script `install-opensim.sh`.

### Running Test

Use `osimperf-cli record` subcommand for running benchmark tests, e.g.:

`osimperf-cli record --config tests/RajagopalFreeFall/osimperf-test.conf --iter 10`

will run the benchmark configured by `osimperf-test.conf` for $10$ times, and record the time to complete the benchmark.

### List Artefacts

Use `osimperf-cli ls` subcommand to list artefacts generated by the tool, e.g.

`osimperf-cli ls --results my-results-directory`

will list all results generated by the `record` command in the given directory.

### Git Log Helper

Use `osimperf-cli log` to simplify listing commits to checkout.
