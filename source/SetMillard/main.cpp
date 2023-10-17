#include <Actuators/Millard2012EquilibriumMuscle.h>
#include <OpenSim/OpenSim.h>

#include <iostream>
#include <stdexcept>
#include <vector>

int set_millard(std::string& model_filename, double fiber_damping) {
    std::cout << "Reading model from: " << model_filename << "\n";
    OpenSim::Model model = OpenSim::Model(model_filename);

    for (OpenSim::Millard2012EquilibriumMuscle& muscle: model.updComponentList<OpenSim::Millard2012EquilibriumMuscle>()) {
        double prev_fiber_damping = muscle.get_fiber_damping();
        muscle.set_fiber_damping(fiber_damping);
        std::cout << "Set muscle " << muscle.getName() << " fiber_damping from "<< prev_fiber_damping << " to " << muscle.get_fiber_damping() << "\n";
    }

    std::cout << "Writing model to: " << model_filename << "\n";
    model.print(model_filename);

    return 0;
}

int check_fiber_damping(std::string& model_filename, double fiber_damping) {
    OpenSim::Model model = OpenSim::Model(model_filename);

    for (OpenSim::Millard2012EquilibriumMuscle& muscle: model.updComponentList<OpenSim::Millard2012EquilibriumMuscle>()) {
        std::cout << "Verifying muscle " << muscle.getName() << " fiber_damping "<< muscle.get_fiber_damping() << "\n";
        if (muscle.get_fiber_damping() != fiber_damping) {
            throw std::runtime_error("failed to set fiber damping");
        }
    }

    return 0;
}

int do_main(int argc, char *argv[]) {
    std::vector<std::string> args(argv + 1, argv + argc);
    std::string model_filename = args[0];
    double fiber_damping = std::stod(args[1]);
    set_fiber_damping(model_filename, fiber_damping);
    check_fiber_damping(model_filename, fiber_damping);
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
