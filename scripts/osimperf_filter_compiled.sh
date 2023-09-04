#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

# ==============================================================
# ======================== Config ==============================
# ==============================================================

# Folder layout.
ARCHIVE=$1

# archive/versionXXX/install

# Go over list and check in archive if:
#	- folder exists: opensim-core-DATA-COMMIT
#	- run install/bin/opensim-cmd --version and check against obtained version

while IFS= read -r line; do
	date_string=$(echo "$line" | awk -F, '{print $1}')
	hash_string=$(echo "$line" | awk -F, '{print $2}')

    folder="$ARCHIVE/opensim-core-$date_string-$hash_string"
    opensim_cmd="$folder/install/bin/opensim-cmd"
    if [[ ! -e "$opensim_cmd" ]]; then
        echo $line
        continue
    fi

    compiled_version=$opensim_cmd --version
    if [[ ! "$compiled_version" = "$hash_string" ]]; then
        echo $line
        continue
    fi
done

# End of the script
