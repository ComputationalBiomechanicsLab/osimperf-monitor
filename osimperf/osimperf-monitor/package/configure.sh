#!/bin/bash
# Run from parent directory of package.

package="osimperf-monitor"

if [ -d "$package/usr" ]; then rm -rf "$package/usr"; fi
mkdir -p "$package/usr"
mkdir -p "$package/usr/bin"

cargo install \
	--bin $package\
	--path "$OSIMPERF_HOME/osimperf/$package" \
	--root "$package/usr" \
	--no-track

if [ -d "$package/etc" ]; then rm -rf "$package/etc"; fi
mkdir -p "$package/etc"
mkdir -p "$package/etc/systemd"
mkdir -p "$package/etc/systemd/system"

read -r -d '' systemd_service <<EOF
[Unit]
Description=OpenSim Performance Monitor Service.
StartLimitIntervalSec=300
StartLimitBurst=5

[Service]
User=$USER
Environment="PATH=$PATH"
Type=simple
ExecStart=/usr/bin/$package --home $OSIMPERF_HOME
Restart=on-failure
RestartSec=10s
Nice=-10
IOSchedulingClass=realtime
IOSchedulingPriority=0

[Install]
WantedBy=multi-user.target
EOF

# EnvironmentFile=/home/username/.bashrc
# Environment="HOME=$HOME"

echo "$systemd_service" > "$package/etc/systemd/system/$package.service"
