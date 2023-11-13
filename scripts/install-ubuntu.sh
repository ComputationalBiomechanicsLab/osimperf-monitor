#!/bin/bash

# This script will install osimperf-monitor and opensim-core's required dependencies on this machine.
# osimperf-monitor will be installed as a systemd service.

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

# Opensim core dependencies
sudo apt-get update && sudo apt-get install --yes \
	autoconf\
	automake\
	autotools-dev\
	build-essential\
	byacc\
	cmake\
	doxygen\
	freeglut3-dev\
	gfortran\
	git\
	liblapack-dev\
	libopenblas-dev\
	libpcre3\
	libpcre3-dev\
	libssl-dev\
	libtool\
	libxi-dev\
	libxmu-dev\
	ninja-build\
	openjdk-8-jdk\
	patchelf\
	pkg-config\
	python3\
	python3-dev\
	python3-numpy\
	python3-setuptools\
	tmux\
	curl\
	pip\
	swig

# Install grip for markdown display:
pip install grip

# Enable ssh.
# systemctl start ssh
# systemctl enable ssh

# git submodule update --init --recursive

# Install rust.
# echo 1 | $("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")

# # Setup vim: install vimplug
# curl -fLo ~/.vim/autoload/plug.vim --create-dirs \
#     https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

# # Install osimperf.
# ./scripts/install-osimperf-service.sh

# # Install pandas and matplotlib for python.

# # Set to nightly.
# env -C osimperf rustup override set nightly

# Install osimperf-cli.
target="osimperf-cli"
osimperf_root="$PWD"

cargo install \
	--bin $target\
	--path "osimperf/$target" \
	--root "$osimperf_root"

cp "scripts/osimperf-default-install-opensim" "$osimperf_root/bin/osimperf-default-install-opensim"
cp "scripts/csv-plot.py" "$osimperf_root/bin/csv-plot.py"

echo "Use RUST_LOG to set log-level."
echo "Use OPENSIM_MODELS to point to opensim-models directory."

echo "dont forget to add osimperf to PATH:"
echo "PATH=$osimperf_root/bin:\$PATH"
