#!/bin/bash

# This script filters out duplicate dates.

# Input:
#	The script processes input line-by-line, with each line is assumed to
# 	contain: "date hash", e.g.:
# 	"2023/01/29 1234..."
# 	"2023/01/28 5291..."
# 	...
#
# 	It is assumed that most recent commit comes first.

# Output
# Same as input, but with duplicate dates removed.

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

date_string=""
while IFS= read -r line; do
	prev_date="$date_string"
	date_string=$(echo "$line" | awk -F, '{print $1}')

    # Filter out duplicate dates.
    if [ "$prev_date" = "$date_string" ]; then
    	continue
	fi

    # Output the line
    echo "$line"
done

# End of the script
