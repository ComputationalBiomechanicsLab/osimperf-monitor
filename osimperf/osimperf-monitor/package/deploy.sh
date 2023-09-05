#!/bin/bash
# Run from parent directory of package.

package="osimperf-monitor"

# Build the package.
dpkg-deb --build "$package"

# Install the package.
dpkg -i "$package.deb"

# # Enable and start service.
# systemctl enable $package
systemctl stop $package
systemctl daemon-reload
systemctl start $package
systemctl status $package


# Uninstalling...
##!/bin/bash

## Stop and disable the service
#systemctl stop my_service
#systemctl disable my_service

## Remove the systemd service file
#rm /lib/systemd/system/my_service.service

## Delete any other files and directories created by your package
#rm -rf /usr/bin/my_script.sh

## Remove the package
#dpkg -r mypackage

## Optionally, remove any dependencies that were installed only for this package
## apt-get autoremove -y
