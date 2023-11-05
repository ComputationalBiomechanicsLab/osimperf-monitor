# OSimPerf Monitor

## Installing (Ubuntu)

To install dependencies and `osimperf-cli` for Ubuntu:

`./scripts/install-ubuntu.sh` 

# Usage

```bash
# Display help:
osimperf-cli --help
```

Install commands:

```bash
# Install opensim-core, creating a subdirectory in current folder:
osimperf-cli install --opensim path_to_opensim_source

# Or set path to opensim-core globally:
export OSPC_OPENSIM_SRC=path_to_opensim_source
osimperf-cli install

# Change install directory:
osimperf-cli install --root my_install_dir

# Use a custom install script:
osimperf-cli install --installer my_custom_script

# Change build directory:
osimperf-cli install --build my_build_dir
```

Finding things:


```bash
# List installed versions by osimperf found in target dir.
osimperf-cli ls --install dir

# List benchmark config files found in target dir.
osimperf-cli ls --tests dir

# List benchmark result files found in target dir.
osimperf-cli ls --results dir
```

Running benchmarks:

```bash
# Running benchmark 10 times (make sure installed version is on path):
osimperf-cli record --config my_benchmark_file --iter 10

# Or read config files from stdin:
osimperf-cli ls --tests dir | osimperf-cli record --iter 10

# Trigger valgrind:
osimperf-cli ls --tests dir | osimperf-cli record --grind
```

## Benchmarks Config Files

DESCRIPTION HERE

## Plotting Results

A table of results as a markdown file:

`osimperf-cli plot --table "table.md"`

A timeline plot of results:

`osimperf-cli plot --figure`

Which results to include can be filtered:

`osimperf-cli plot --figure --results "$(osimperf-cli ls --results . | grep Rajagopal)"`

## Relevant Environmental Variables

Consider adding these to `.bashrc` to simplify the work:

- `OSPC_OPENSIM_MODELS`: Directory to [opensim-models]() source.
- `OSPC_OPENSIM_SOURCE`: Directory to [opensim-core]() source.
- `OSPC_BUILD_DIR`: To overwrite default build directory.

# Examples

See `examples/README.md` folder.
