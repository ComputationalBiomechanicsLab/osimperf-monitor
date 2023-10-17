#!/bin/bash

# This script will install osimperf-monitor and opensim-core's required dependencies on this machine.
# osimperf-monitor will be installed as a systemd service.

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

# Opensim core dependencies
packages="build-essential cmake autotools-dev autoconf pkg-config automake libopenblas-dev liblapack-dev freeglut3-dev libxi-dev libxmu-dev doxygen python3 python3-dev python3-numpy python3-setuptools git byacc libssl-dev libpcre3 libpcre3-dev libtool gfortran ninja-build patchelf openjdk-8-jdk swig"

# Other useful packages.
packages+=" opensssh-server vim curl tmux"

# For viewing markdown
packages+=" okular okular-extra-backends retext"

sudo apt-get update && sudo apt-get install --yes "$packages"

# Enable ssh.
systemctl start ssh
systemctl enable ssh

git submodule update --init --recursive

# Install rust.
echo 1 | $("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")

# Setup vim: install vimplug
curl -fLo ~/.vim/autoload/plug.vim --create-dirs \
    https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

# Build osimperf from source.
./scripts/build-osimperf.sh

# Install osimperf.
./scripts/install-osimperf-service.sh
