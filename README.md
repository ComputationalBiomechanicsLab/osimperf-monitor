# OSimPerf Monitor

## Installing for Ubuntu

To install dependencies and `osimperf-cli` for Ubuntu:
`./scripts/install-ubuntu.sh` 

# OSimPerf CLI

Use `osimperf-cli --help` to get more info.

## Git Log Helper Command

`osimperf-cli git-checkout --date yyyy-mm-dd --opensim PATH_TO_OPENSIM --clone --branch BRANCH_NAME`

## Install Command

To install currently checked out version of opensim-core.
`osimperf-cli install --opensim PATH_TO_OPENSIM_SOURCE`

Or set `OSPC_OPENSIM_SRC` to the opensim source directory, and run:
`osimperf-cli install`

To use a custom install script:
`osimperf-cli install --installer PATH_TO_SCRIPT`

By default a unique subdirectory is created in the current directory using the commit hash.
To customize the install root directory use:
`osimperf-cli install --path "my_custom_path/custom_dir-%b-%H-%y_%m_%d"`

The default install script uses the build directory `/tmp/osimperf-build-opensim-core`.
This can be changed by setting `OSPC_BUILD_DIR`, or by running:
`osimperf-cli install --build BUILD_DIR`

The branch name is used to create more human readable install-info.
The `install` command will not switch branches, but does verify that the checked out commit is part of the given branch.
By default the branch is set to `main`, but can be changed:
`osimperf-cli install --branch "my_branch_name"`

## Install Info Command

To make sure that the benchmark tests, and other scripts can find the installed version, make sure to prefix the path.
This can be done by:
`for path in $(osimperf-cli --install .); do; export PATH="path:$PATH"; ...`

To verify that the path variable is set, run `osimperf-install-info`.

## Record Command

Assuming the path to `osimperf-install-info` is set correctly, running a benchmark can be done using:
`osimperf-cli record --config CONFIG_FILE`

To collect all `osimperf-test.conf` from the current directory and subdirectories, and run them, ommit the `--config` argument:
`osimperf-cli record`

Which is the same as:
`osimperf-cli record --config "$(osimperf-cli ls --tests .)"`

Overwriting the default number of test iterations:
`osimperf-cli record --iter 10`

Running valgrind:
`osimperf-cli record --grind`

Running visualizer:
`osimperf-cli record --visualize`

To print the executed command:
`osimperf-cli record --print`

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

`OSPC_OPENSIM_MODELS`: Directory to [opensim-models]() source.
`OSPC_OPENSIM_SOURCE`: Directory to [opensim-core]() source.
`OSPC_BUILD_DIR`: To overwrite default build directory.

# Examples

See `examples/README.md` folder.
