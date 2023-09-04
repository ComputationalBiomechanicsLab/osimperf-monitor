#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

# ========================
# ======= CONFIG =========
# ========================

branch="main"
start_date="2023-07-15"
path_to_source="."

# ==========================================
# ======= Check path to repository =========
# ==========================================

if [[ -v path_to_source ]]; then
	>&2 echo "No path to opensim-core source repo specified"
fi

# Check if path to source is the opensim-core repository.
source_remote=$(echo $(git remote -v | grep fetch | awk '{print $2}'))
if [[ "$source_remote" != "https://github.com/opensim-org/opensim-core.git" ]]; then
	>&2 echo "Path to source is not the opensim-core repository."
	exit 1
fi

# Check if we are on the main branch.
if [ "$(git rev-parse --abbrev-ref HEAD)" != "$branch" ]; then
	>&2 echo "Current branch is not $branch"
	exit 1
fi

cd $path_to_source
git remote --update >/dev/null
cd-

# ==========================================
# ======= Get commits per date =============
# ==========================================

# Initialize an empty array to store the formatted dates
commit_dates=()
commit_hashes=()

# Get the list of commit dates and format them.
while IFS= read -r line; do
	# Each line is formatted as:
	# line = "Date:   Thu Jul 27 15:22:18 2023 +0200"

	# Extract the date string from the "Date:" line
	date_string=$(echo "$line" | awk '{print $4, $3, $6}')
	# Convert the date to the desired format (YYYY/MM/DD)
	formatted_date=$(date -d "$date_string" "+%Y/%m/%d")
	# Append the formatted date to the array
	commit_dates+=("$formatted_date")

done <<< "$(git log $branch --after="$start_date" | grep Date)"

# Get the list of commit hashes.
while IFS= read -r line; do
	# Each line is formatted as:
	# line = "commit: 1234..."

	hash_string=$(echo "$line" | awk '{print $2}')
	commit_hashes+=("$hash_string")

done <<< "$(git log --all --after="$start_date" | grep commit)"

delimiter=","
for i in ${!commit_dates[@]}; do
	echo "${commit_dates[$i]}$delimiter${commit_hashes[$i]}"
done
