# OSimPerf Monitor
(Should be called OSimPerf-Collector)

`scripts/install-ubuntu.sh` should install dependencies for Ubuntu.

`scripts/build-osimperf.sh` builds osimperf-binaries from source. Binaries will be placed in `bin/`

`scripts/install-osimperf-service.sh` installs the service `osimperf-monitor` and installs the binary `osimperf-tui`.

Running `osimperf-monitor --help` should be helpful. These binaries require `--home this-repo-path`, i.e. they need the path to this repo.
Running `osimperf-tui --home this-repo-path` visualizes the progress of the monitor.
When ready, run `scripts/print-csv.sh` to plot the pref scatter plots.

Relevant folders:

- `tests/` contains the defined benchmarks.
- `archive/` will be where all compiled versions will be installed.
- `results/` will be where all benchmark results will be written.
