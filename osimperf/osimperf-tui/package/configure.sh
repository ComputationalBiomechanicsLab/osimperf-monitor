#!/bin/bash
# Run from parent directory of package.

package="osimperf-tui"

if [ -d "$package/usr" ]; then rm -rf "$package/usr"; fi
mkdir -p "$package/usr"
mkdir -p "$package/usr/bin"

cargo install \
	--bin $package\
	--path "$OSIMPERF_HOME/osimperf/$package" \
	--root "$package/usr" \
	--no-track

echo "Be sure to add alias to .bashrc: \"alias osimperf-tui='osimperf-tui --path $OSIMPERF_HOME'\""
