#pragma once

#include <cmath>
#include <stdexcept>
#include <string>
#include <vector>

namespace Hopper
{

    //==========================================================================
    // Command line argument parsing.
    //==========================================================================
    // An optional argument to this executable:
    //    noVisualizer -- [Optional] Do not show visualizer. Default
    //                       behavior is to show visualizer.
    struct Config
    {
        explicit Config(const std::vector<std::string>& args)
        {
            for (auto it = args.begin(); it < args.end(); ++it) {
                if (*it == "--visualize") {
                    showVisualizer = true;
                }
                if (*it == "--accuracy") {
                    accuracy = std::stod(*++it);
                }
                if (*it == "--damping") {
                    model.fiberDamping = std::stod(*++it);
                }
                if (*it == "--final-time") {
                    finalTime = std::stod(*++it);
                }
                // Path to where model is written.
                if (*it == "--model-xml") {
                    modelPath = *++it;
                }
                // Path to where forward tool setup is written.
                if (*it == "--setup-xml") {
                    setupPath = *++it;
                }
                if (*it == "--results-dir") {
                    resultsDir = *++it;
                }
                if (*it == "--print-RUSTCSVPLOT") {
                    print_rustcsvplot = true;
                }

            }
            if (!setupPath.empty()) {
                if (resultsDir.empty()) {
                    throw std::runtime_error("Please provide --results-dir argument for the setup-xml");
                }
            }
        }

        Config(int argc, char* argv[]) : Config({argv + 1, argv + argc})
        {}

        struct ModelConfig
        {
            struct ActivationPoint
            {
                double dt;
                double value;
            };
            std::vector<ActivationPoint> activationSignal{
                {2., 0.01},
                {2.,   1.},
                {2.,  0.3},
                {1., 0.75},
            };

            double fiberDamping   = NAN;
            double maxContraction = NAN;
        } model;

        double accuracy         = NAN;
        double finalTime        = 10.;
        double reporterTimeStep = 1e-2;

        std::string modelPath;
        std::string setupPath;
        std::string resultsDir;

        bool showVisualizer{false};
        bool print_rustcsvplot{false};

        bool writeModel() const {
            return !modelPath.empty();
        }

        bool writeSetup() const {
            return !setupPath.empty();
        }
    };

} // namespace Hopper
