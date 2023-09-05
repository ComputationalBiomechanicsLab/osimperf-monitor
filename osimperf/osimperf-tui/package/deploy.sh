#!/bin/bash
# Run from parent directory of package.

package="osimperf-tui"

# Build the package.
dpkg-deb --build "$package"

# Install the package.
dpkg -i "$package.deb"
