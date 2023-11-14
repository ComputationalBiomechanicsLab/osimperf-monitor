/* -------------------------------------------------------------------------- *
 *                     OpenSim:  exampleHopperDevice.cpp                      *
 * -------------------------------------------------------------------------- *
 * The OpenSim API is a toolkit for musculoskeletal modeling and simulation.  *
 * See http://opensim.stanford.edu and the NOTICE file for more information.  *
 * OpenSim is developed at Stanford University and supported by the US        *
 * National Institutes of Health (U54 GM072970, R24 HD065690) and by DARPA    *
 * through the Warrior Web program.                                           *
 *                                                                            *
 * Copyright (c) 2005-2017 Stanford University and the Authors                *
 * Author(s): Chris Dembia, Shrinidhi K. Lakshmikanth, Ajay Seth,             *
 *            Thomas Uchida                                                   *
 *                                                                            *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may    *
 * not use this file except in compliance with the License. You may obtain a  *
 * copy of the License at http://www.apache.org/licenses/LICENSE-2.0.         *
 *                                                                            *
 * Unless required by applicable law or agreed to in writing, software        *
 * distributed under the License is distributed on an "AS IS" BASIS,          *
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.   *
 * See the License for the specific language governing permissions and        *
 * limitations under the License.                                             *
 * -------------------------------------------------------------------------- */

/* This example demonstrates some of the new features of the OpenSim 4.0 API.
The Component architecture allows us to join sub-assemblies to form larger
Models, with information flowing between Components via Inputs, Outputs, and
Sockets. For more information, please refer to the Component documentation.

This interactive example consists of three steps:
  Step 1. Build and simulate a single-legged hopping mechanism.
  Step 2. Build an assistive device and test it on a simple testbed.
  Step 3. Connect the device to the hopper to increase hop height.

To start working through this example, go to run() at the bottom of this file.
From there, you will be directed to specific files and methods in this project
that need to be completed. Now, hop to it! */

#include "config.h"
#include "defineDeviceAndController.h"
#include <Common/ComponentOutput.h>
#include <Common/Reporter.h>
#include <Simulation/Model/PhysicalFrame.h>
#include <Simulation/SimbodyEngine/SliderJoint.h>
#include <Tools/ForwardTool.h>
#include <algorithm>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

#include <OpenSim/OpenSim.h>

static const double SIGNAL_GEN_CONSTANT{0.33};
static const double REPORTING_INTERVAL{0.2};

static const std::string testbedAttachment1{"ground"};
static const std::string testbedAttachment2{"bodyset/load"};

static const std::string thighAttachment{"bodyset/thigh/deviceAttachmentPoint"};
static const std::string shankAttachment{"bodyset/shank/deviceAttachmentPoint"};

namespace OpenSim
{

    // Forward declarations for methods used below.
    Model buildHopper(
        bool showVisualizer, const Hopper::Config::ModelConfig&
                                 config);    // defined in buildHopperModel.cpp
    Model buildTestbed(bool showVisualizer); // defined in buildTestbedModel.cpp
    Device* buildDevice();                   // defined in buildDevice.cpp

    //------------------------------------------------------------------------------
    // Attach the device to any two PhysicalFrames in a model.
    // [Step 2, Task D]
    //------------------------------------------------------------------------------
    void connectDeviceToModel(
        OpenSim::Device& device, OpenSim::Model& model,
        const std::string& modelFrameAname, const std::string& modelFrameBname)
    {
        // Add the device to the model.
        model.addComponent(&device);

        /* model.printSubcomponentInfo(); */

        // Get writable references to the "anchor" joints in the device.
        auto& anchorA = device.updComponent<WeldJoint>("anchorA");
        auto& anchorB = device.updComponent<WeldJoint>("anchorB");

        // Connect anchor points to frames on model.
        const auto& modelFrameA =
            model.getComponent<PhysicalFrame>(modelFrameAname);
        const auto& modelFrameB =
            model.getComponent<PhysicalFrame>(modelFrameBname);

        anchorA.connectSocket_parent_frame(modelFrameA);
        anchorB.connectSocket_parent_frame(modelFrameB);

        // Configure the device to wrap over the patella (if one exists; there
        // is no patella in the testbed).
        const std::string patellaPath("thigh/patellaFrame/patella");
        if (model.hasComponent<WrapCylinder>(patellaPath)) {
            auto& cable = model.updComponent<PathActuator>("device/cableAtoB");
            auto& wrapObject = model.updComponent<WrapCylinder>(patellaPath);
            cable.updGeometryPath().addPathWrap(wrapObject);
        }
    }

    //------------------------------------------------------------------------------
    // Add a ConsoleReporter to the hopper model to display variables of
    // interest. [Step 1, Task B]
    //------------------------------------------------------------------------------
    template<typename ReporterType>
    void addReporterToHopper(
        Model& hopper, std::string name, double reporterInterval,
        bool logAll = false)
    {
        // Create a new ConsoleReporter. Set its name and reporting interval.
        auto reporter = new ReporterType();
        reporter->setName(name);
        reporter->set_report_time_interval(reporterInterval);

        // Loop through the desired device outputs and add them to the reporter.
        if (logAll) { // TODO this could be cleaner...
            for (const Component& component : hopper.getComponentList()) {
                for (const std::string& oName : component.getOutputNames()) {
                    const AbstractOutput& output = component.getOutput(oName);
                    if (output.getTypeName() != "double")
                        continue;
                    reporter->addToReport(output);
                }
            }
        } else {
            for (const Component& component :
                 hopper.getComponentList<Muscle>()) {
                for (const std::string& oName : component.getOutputNames()) {
                    const AbstractOutput& output = component.getOutput(oName);
                    if (output.getTypeName() != "double")
                        continue;
                    reporter->addToReport(output);
                }
            }
        }

        // Add the reporter to the hopper.
        hopper.addComponent(reporter);
    }

    //------------------------------------------------------------------------------
    // Add a SignalGenerator to a device.
    // [Step 2, Task E]
    //------------------------------------------------------------------------------
    void addSignalGeneratorToDevice(Device& device)
    {
        auto* signalGen = new SignalGenerator();
        signalGen->setName("signalGenerator");

        // Try changing the constant value and/or the function (e.g., try a
        // LinearFunction).
        signalGen->set_function(Constant(SIGNAL_GEN_CONSTANT));
        device.addComponent(signalGen);

        // TODO: Connect the signal generator's output signal to the
        // controller's
        //       activation input.
        AbstractInput& activationInput =
            device.updComponent<Controller>("controller")
                .updInput("activation");
        activationInput.connect(signalGen->getOutput("signal"));
    }

    //------------------------------------------------------------------------------
    // Add a ConsoleReporter to a model for displaying outputs from a device.
    //------------------------------------------------------------------------------
    void addDeviceConsoleReporterToModel(
        Model& model, Device& device,
        const std::vector<std::string>& deviceOutputs,
        const std::vector<std::string>& deviceControllerOutputs,
        const std::vector<std::string>& modelOutputs)
    {
        // Create a new ConsoleReporter. Set its name and reporting interval.
        auto reporter = new ConsoleReporter();
        reporter->setName(
            model.getName() + "_" + device.getName() + "_results");
        reporter->set_report_time_interval(REPORTING_INTERVAL);

        // Loop through the desired device outputs and add them to the reporter.
        for (auto thisOutputName : deviceOutputs)
            reporter->addToReport(device.getOutput(thisOutputName));

        for (auto thisOutputName : deviceControllerOutputs)
            reporter->addToReport(
                device.getComponent("controller").getOutput(thisOutputName));
        for (const Component& component : model.getComponentList()) {
            for (const std::string& oName : component.getOutputNames()) {
                for (auto thisOutputName : deviceControllerOutputs) {
                    if (oName != thisOutputName) {
                        continue;
                    }
                    reporter->addToReport(component.getOutput(oName));
                }
            }
        }
        // Add the reporter to the model.
        model.addComponent(reporter);
    }

    void printTableToPlot(std::ostream& os, const TableReporter& reporter)
    {
        const auto& table  = reporter.getTable();
        const auto& matrix = table.getMatrix();
        for (int i = 0; i < matrix.nrow(); i++) {
            double time = table.getIndependentColumn()[i];
            for (int j = 0; j < matrix.ncol(); j++) {
                const auto& label = table.getColumnLabel(j);
                double value      = matrix.row(i)[j];
                os << "RUSTCSVPLOT," << label << "," << time << "," << value
                   << std::endl;
            }
        }
    }

} // namespace OpenSim

//------------------------------------------------------------------------------
// START HERE! Toggle "if (false)" to "if (true)" to enable/disable each step in
// the exercise. The project should execute without making any changes (you
// should see the unassisted hopper hop slightly).
//------------------------------------------------------------------------------
void run(const Hopper::Config& config)
{
    using namespace OpenSim;
    // Build the hopper and device.
    auto assistedHopper = buildHopper(config.showVisualizer, config.model);
    auto kneeDevice     = buildDevice();

    // Connect the device to the hopper.
    connectDeviceToModel(
            *kneeDevice, assistedHopper, thighAttachment, shankAttachment);

    // Use the vastus muscle's activation as the control signal for the
    // device.
    kneeDevice->updComponent("controller")
        .updInput("activation")
        .connect(assistedHopper.getComponent("forceset/vastus")
                .getOutput("activation"));

    // List the device outputs we wish to display during the simulation.
    std::vector<std::string> kneeDeviceOutputs{"tension", "height"};
    std::vector<std::string> deviceControllerOutputs{"myo_control"};
    std::vector<std::string> modelOutputs{"activation"};

    // Add a ConsoleReporter to report deviceOutputs.
    /* assistedHopper.printOutputInfo(true); */
    /* addDeviceConsoleReporterToModel( */
    /*     assistedHopper, *kneeDevice, kneeDeviceOutputs, */
    /*     deviceControllerOutputs, modelOutputs); */

    std::string reporterName = "tableReporter";
    addReporterToHopper<TableReporter>(
            assistedHopper, reporterName, config.reporterTimeStep);

    // Create the system, initialize the state, and simulate.
    SimTK::State& initState = assistedHopper.initSystem();
    /* simulate(assistedHopper, sHD, config.finalTime); */

    if (config.writeSetup()) {
        // Create tool and write setup.xml file.
        assistedHopper.print(config.modelPath);
        ForwardTool tool;
        tool.setName("ForwardIntegration");
        tool.setModelFilename(config.modelPath);
        /* tool.setManager(manager); */
        tool.setFinalTime(config.finalTime);
        tool.setResultsDir(config.resultsDir);
        tool.setErrorTolerance(config.accuracy);
        /* tool.updControllerSet().adoptAndAppend(new
         * PrescribedController("exampleHangingMuscle_controls.sto")); */

        tool.print(config.setupPath);
    } else {
        Manager manager(assistedHopper);
        manager.setIntegratorMethod(
                Manager::IntegratorMethod::RungeKuttaMerson); // Does this even do
        manager.setIntegratorAccuracy(config.accuracy);

        manager.initialize(initState);
        manager.integrate(config.finalTime);

        std::cout << "Attempted steps: "
            << manager.getIntegrator().getNumStepsAttempted()
            << std::endl;
        std::cout << "Realized steps: "
            << manager.getIntegrator().getNumStepsTaken() << std::endl;
    }

    if (config.print_rustcsvplot) {
        printTableToPlot(
                std::cout,
                assistedHopper.getComponent<TableReporter>(reporterName));
    }
};

int main(int argc, char* argv[])
{
    Hopper::Config config(argc, argv);

    try {
        run(config);
    } catch (const std::exception& ex) {
        std::cout
            << "Hopper Example Failed to run due to the following Exception: "
            << ex.what() << std::endl;
        return 1;
    }

    return 0;
}
