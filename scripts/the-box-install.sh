#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail


packages="opensssh-server"

# Post installation
systemctl start ssh
systemctl enable ssh

# To get the ip address of the mini pc:
ip a

git submodule update --init --recursive
