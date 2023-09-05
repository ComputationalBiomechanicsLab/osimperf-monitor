#!/bin/bash
# Run from parent directory of package.

package="osimperf-tui"

mkdir -p "$package/usr"
mkdir -p "$package/usr/bin"

cargo install \
	--bin $package\
	--path "$OSIMPERF_HOME/osimperf/$package" \
	--root "$package/usr"
