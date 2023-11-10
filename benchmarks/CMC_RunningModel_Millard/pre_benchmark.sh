#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start RunningCMC (Opensim version = $(opensim-cmd --version))."

models="$OSPC_OPENSIM_MODELS"

if [ -z $models ]; then
	echo "ERROR: no models dir found"
	exit 1
fi

cp ../CMC_RunningModel/runningModel_CMC_Actuators.xml .
cp ../CMC_RunningModel/runningModel_CMC_Tasks.xml .
cp ../CMC_RunningModel/runningModel_CMC_test.osim .
cp ../CMC_RunningModel/runningModel_ControlConstraints.xml .
cp ../CMC_RunningModel/runningModel_GRF.xml .
cp ../CMC_RunningModel/runningModel_GRF_data.mot .
cp ../CMC_RunningModel/runningModel_Kinematics_q.sto .
cp ../CMC_RunningModel/runningModel_Setup_CMC_test.xml .

../CMC_RunningModel/pre_benchmark.sh

sed -i -e s/'Thelen2003Muscle'/'Millard2012EquilibriumMuscle'/g runningModel_CMC_test.osim

if [ ! -z $(cat runningModel_CMC_test.osim | grep Thelen2003Muscle) ]; then
	echo "Failed to remove Thelen2003Muscle muscle from $PWD/runningModel_CMC_test.osim"
	exit 1
fi

SetFiberDamping runningModel_CMC_test.osim 0.01

# sed -i -e s/'fiber_damping>0.1<\/fiber_damping'/'fiber_damping>0.01<\/fiber_damping'/g runningModel_CMC_test.osim
