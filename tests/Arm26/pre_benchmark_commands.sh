#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

cp -r "$OSIMPERF_MODELS/Tutorials/Computed_Muscle_Control/OutputReference/StaticOptimization/NonphysiologicalResults $OSIMPERF_CONTEXT"
cp -r "$OSIMPERF_MODELS/Geometry $OSIMPERF_CONTEXT"
cp "$OSIMPERF_MODELS/Tutorials/Computed_Muscle_Control/OutputReference/arm26.osim $OSIMPERF_CONTEXT/arm26.osim"
cp "$OSIMPERF_MODELS/Tutorials/Computed_Muscle_Control/OutputReference/ForwardDynamics/arm26_Setup_Forward.xml $OSIMPERF_CONTEXT/arm26_setup.xml"

"$OSIMPERF_SETUP/../../source/install_hotfix.sh"
