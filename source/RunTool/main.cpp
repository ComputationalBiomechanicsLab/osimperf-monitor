#include <OpenSim/OpenSim.h>
#include <OpenSim/ExampleComponents/RegisterTypes_osimExampleComponents.h>

#include <iostream>
#include <stdexcept>
#include <vector>

int do_main(int argc, char *argv[]) {
    RegisterTypes_osimExampleComponents();
    std::vector<std::string> args(argv + 1, argv + argc);
    std::string tool_filename = args[0];
    std::string model_filename = args[1];

    /* std::cout << "Reading model from: " << model_filename << "\n"; */
    OpenSim::Model model = OpenSim::Model(model_filename);

    /* std::cout << "Enable visualizer.\n"; */
    /* model.setUseVisualizer(true); */

    /* std::cout << "Reading tool from: " << tool_filename << "\n"; */
    OpenSim::ForwardTool tool(tool_filename);
    tool.setName("ForwardIntegration");
    tool.setModel(model);
    tool.run();

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
