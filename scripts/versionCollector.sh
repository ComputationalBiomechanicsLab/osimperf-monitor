# Notes:
# This program will not handle commits that break compilation very well.
# This might require a redesign of how commits/dates are stored.
# For now we could go with marking a date+commit as broken, and skipping it later on.
# If this turns out to be a real problem we could improve.

# ========================
# ======= CONFIG =========
# ========================

# Input:
# --start-date YYYY-MM-DD
# --source PATH-TO-OPENSIM-CORE-SOURCE
# If source is not provided it will try the url.
start_date="2023-07-15"
path_to_source=""

# Do a remote update

# ========================
# ======= OUTPUT =========
# ========================

# Outputs comma seperated date and commit hash, from most recent to least:
# YYYY-MM-DD,HASH

# current_date=$(date +"%Y-%m-%d")

# === Get Commits per date ===

# Initialize an empty array to store the formatted dates
commit_dates=()
commit_hashes=()

path_to_source="."
if [[ -v path_to_source ]]; then
	# Get list of dates and commits from opensim-core source.

	# Get the list of commit dates and format them
	while IFS= read -r line; do
		# Each line is formatted as:
		# line = "Date:   Thu Jul 27 15:22:18 2023 +0200"

		# Extract the date string from the "Date:" line
		date_string=$(echo "$line" | awk '{print $4, $3, $6}')
		# Convert the date to the desired format (YYYY/MM/DD)
		formatted_date=$(date -d "$date_string" "+%Y/%m/%d")
		# Append the formatted date to the array
		commit_dates+=("$formatted_date")
	done <<< "$(git log --all --after="$start_date" | grep Date)"

	# Get the list of commit dates and format them
	while IFS= read -r line; do
		# Each line is formatted as:
		# line = "commit: 1234..."
		hash_string=$(echo "$line" | awk '{print $2}')
		commit_hashes+=("$hash_string")
	done <<< "$(git log --all --after="$start_date" | grep commit)"

	echo "${#commit_dates[@]}"
	echo "${#commit_hashes[@]}"

	exit 1

else
	>&2 echo "Dont use url yet"
	exit 1
	# Get list of dates and commits from opensim-core on github.
	owner="opensim-org"
	repo="opensim-core"
	api_url="https://api.github.com/repos/$owner/$repo/commits?since=$start_date" # &until=$current_date"
	commits_since=$(echo $(curl -s $api_url))
	# cat "commitsSinceResponse.json"
	# commits_since=$(cat $commits_since_response_file)
fi

# === Get Commits per date ===


# Store commit hashes and dates in an array.
read -rd "" -a commit_hash_arr <<< $(echo "$response" | jq -r ".[] | .sha")
read -rd "" -a commit_timestamp_arr <<< $(echo "$response" | jq -r ".[] | .commit.committer.date")

# Strip timestamp from dates, leaving "YYYY-MM-DD" format.
for i in ${!commit_timestamp_arr[@]}; do
	commit_date_arr[i]=$(date -d "${commit_timestamp_arr[i]}" +%Y-%m-%d)
done

# Keep only last commit per date.
for i in ${!commit_date_arr[@]}; do
	# If previous commit has same date, we skip it.
	if (( i > 0 )); then
		if [ "${commit_date_arr[$i]}" = "${commit_date_arr[$(($i-1))]}" ]; then
			continue
		fi
	fi
	echo "${commit_date_arr[$i]},${commit_hash_arr[$i]}"
done
