#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

NAME="RajagopalFreeFall"
echo
echo "Start $NAME (Opensim version = $(opensim-cmd --version))."

models="$OSPC_OPENSIM_MODELS"

cp -r $models/Geometry .
cp -r $models/Models/RajagopalModel/Geometry .
cp -r $models/Models/RajagopalModel/Rajagopal2015.osim .

sed -i -e s/'fiber_damping>0.01<\/fiber_damping'/'fiber_damping>0.1<\/fiber_damping'/g Rajagopal2015.osim

if [ ! -z $(cat Rajagopal2015.osim | grep fiber_damping | grep "0.01") ]; then
	echo "Failed to setup $NAME benchmark."
	echo "Failed to set fiber damping to 0.1"
	exit 1
fi
