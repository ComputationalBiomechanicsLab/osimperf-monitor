#!/bin/bash
set -ueo pipefail

packages="opensssh-server"

# Post installation
systemctl start ssh
systemctl enable ssh

# To get the ip address of the mini pc:
ip a
