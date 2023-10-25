#!/bin/bash
set -eo pipefail

for install_info in $(osimperf-cli ls --install "$PWD"); do
	PATH="$($install_info path):$PATH" env -C "source" "./install.sh"
done
