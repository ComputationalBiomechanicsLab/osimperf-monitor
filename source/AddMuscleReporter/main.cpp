#include <OpenSim/OpenSim.h>
#include <OpenSim/ExampleComponents/RegisterTypes_osimExampleComponents.h>

#include <iostream>
#include <stdexcept>
#include <vector>

int do_main(int argc, char *argv[]) {
    RegisterTypes_osimExampleComponents();
    std::vector<std::string> args(argv + 1, argv + argc);
    std::string model_filename = args[0];

    std::cout << "Reading model from: " << model_filename << "\n";
    OpenSim::Model model = OpenSim::Model(model_filename);

    OpenSim::TableReporter* reporter = new OpenSim::TableReporter();
    reporter->setName("muscleReporter");

    if (argc == 3) {
        double reporterInterval= std::stod(args[1]);
        std::cout << "Set reporter interval to " << reporterInterval << "\n";
        reporter->set_report_time_interval(reporterInterval);
    }

    for (const OpenSim::Component& component :
            model.getComponentList<OpenSim::Muscle>()) {
        for (const std::string& oName : component.getOutputNames()) {
            const OpenSim::AbstractOutput& output = component.getOutput(oName);
            if (output.getTypeName() != "double")
                continue;
            std::cout << "    adding output: " << oName << "\n";
            reporter->addToReport(output);
        }
    }
    std::cout << "Writing model to: " << model_filename << "\n";
    model.addComponent(reporter);
    model.finalizeConnections();
    model.print(model_filename);

    return 0;
}

int main(int argc, char *argv[]) {

    try {
        do_main(argc, argv);
    } catch (const std::exception &ex) {
        std::cout << "main failed with exception: " << ex.what() << std::endl;
        return 1;
    }

    return 0;
}
