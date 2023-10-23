#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start ToyLanding pre-benchmark script."
echo "    Opensim version = $(opensim-cmd --version)"

models="../opensim-models"

cp -r $models/Geometry .
cp $models/Models/ToyLanding/* .
cp $models/Tutorials/Prevention_of_Ankle_Injury/ActiveAFO_Controls.xml .
