#include <Actuators/DeGrooteFregly2016Muscle.h>
#include <Moco/osimMoco.h>
#include <OpenSim/Actuators/ModelOperators.h>
#include <OpenSim/Common/Adapters.h>
#include <OpenSim/OpenSim.h>
#include <Simulation/Model/Muscle.h>

#include <algorithm>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

using namespace OpenSim;

using SimTK::Inertia;
using SimTK::Vec3;

// Basic command line argument parsing.
struct Args {
	bool _visualize = false;
	bool _deGroote = false;
	bool _ignore_tendon_compliance = false;
	std::string _prefix = "";
	double _excitationInitial = 0.1;
	double _excitationFinal = 1.;
	double _muscleFiberDamping = 0.1;
	double _vMax = NAN;
	double _accuracy = 1e-3;
	double _sim_time = 10.;
	double _reporter_dt = 1e-3;
	double _integrator_method = NAN;

	Args(int argc, char *argv[]) {
		std::vector<std::string> args(argv + 1, argv + argc);
		for (auto it = args.begin(); it < args.end(); it++) {
			// Visualize flag.
			this->_visualize |= *it == "-v";
			// Muscle model flag.
			this->_deGroote |= *it == "-g";
			// Ignore tendon compliance.
			this->_ignore_tendon_compliance |= *it == "-ignore-tendon-compliance";
			// Muscle fiber damping.
			if (*it == "-d") {
				this->_muscleFiberDamping = std::stod(*++it);
			}
			// Muscle max contraction velocity.
			if (*it == "-vmax") {
				this->_vMax = std::stod(*++it);
			}
			// Log-prefix flag.
			if (*it == "-p") {
				this->_prefix = *++it;
			}
			// Accuracy
			if (*it == "-a") {
				this->_accuracy = std::stod(*++it);
			}
			// Initial excitation.
			if (*it == "-ai") {
				this->_excitationInitial = std::stod(*++it);
			}
			// Final excitation.
			if (*it == "-af") {
				this->_excitationFinal = std::stod(*++it);
			}
			// Simulation time.
			if (*it == "-s") {
				this->_sim_time = std::stod(*++it);
			}
			// Reporter dt.
			if (*it == "-r") {
				this->_reporter_dt = std::stod(*++it);
			}
			// Integrator method
			if (*it == "-i") {
				this->_integrator_method = std::stod(*(++it));
			}
		};
	}

	using StrMethodPair =
		std::pair<std::string, OpenSim::Manager::IntegratorMethod>;

	const StrMethodPair INTEGRATORS[7] = {
		StrMethodPair("ExplicitEuler",
					  OpenSim::Manager::IntegratorMethod::ExplicitEuler),
		StrMethodPair("RungeKutta2",
					  OpenSim::Manager::IntegratorMethod::RungeKutta2),
		StrMethodPair("RungeKutta3",
					  OpenSim::Manager::IntegratorMethod::RungeKutta3),
		StrMethodPair("RungeKuttaFeldberg",
					  OpenSim::Manager::IntegratorMethod::RungeKuttaFeldberg),
		StrMethodPair("RungeKuttaMerson",
					  OpenSim::Manager::IntegratorMethod::RungeKuttaMerson),
		StrMethodPair("SemiExplicitEuler2",
					  OpenSim::Manager::IntegratorMethod::SemiExplicitEuler2),
		StrMethodPair("Verlet", OpenSim::Manager::IntegratorMethod::Verlet)};

	StrMethodPair get_integrator_method() const {
		int index =
			std::isnan(this->_integrator_method) ? 4 : this->_integrator_method;
		return INTEGRATORS[index];
	}

	void set_manager_args(Manager &manager) {
		if (!std::isnan(this->_integrator_method)) {
			std::cout << "integrator method = " << get_integrator_method().first
					  << "\n";
			manager.setIntegratorMethod(this->get_integrator_method().second);
		}
		if (!std::isnan(this->_accuracy)) {
			std::cout << "integrator Accuracy = " << _accuracy << "\n";
			manager.setIntegratorAccuracy(this->_accuracy);
		}
	}
};

int main(int argc, char *argv[]) {
	// Clap for convenience.
	Args args(argc, argv);

	// Create new model.
	Model model = Model();
	model.setName("Blips");
	std::string prefix = args._prefix;

	// Create two links, each with a mass of 1 kg, center of mass at the body's
	// origin, and moments and products of inertia of zero.
	double mass = 1.;
	auto humerus = new Body("humerus", mass, Vec3(0), Inertia(0));
	auto radius = new Body("radius", mass, Vec3(0), Inertia(0));

	// Connect the bodies with pin joints. Assume each body is 1 m long.
	auto shoulder = new PinJoint("shoulder",
								 model.getGround(), // Parent body
								 Vec3(0, 2, 0),		// Location in parent
								 Vec3(0),			// Orientation in parent
								 *humerus,			// Child body
								 Vec3(0, 1, 0),		// Location in child
								 Vec3(0)			// Orientation in child
	);
	auto elbow = new PinJoint("elbow", *humerus, Vec3(0), Vec3(0), *radius,
							  Vec3(0, 1, 0), Vec3(0));

	// Add a muscle that flexes the elbow.
	double maxIsometricForce = 200;	 // N
	double optimalFiberLength = 0.6; // m
	double tendonSlackLength = 0.55; // m
	double pennationAngle = 0.0;	 // rad

	std::cout << "Muscle properties:" << std::endl;
	std::cout << "    maxIsometricForce = " << maxIsometricForce << std::endl;
	std::cout << "    optimalFiberLength = " << optimalFiberLength << std::endl;
	std::cout << "    tendonSlackLength = " << tendonSlackLength << std::endl;
	std::cout << "    pennationAngle = " << pennationAngle << std::endl;
	std::cout << "    ratio tendonSlackLength to optimalFiberLength = " << tendonSlackLength / optimalFiberLength << std::endl;

	Muscle *biceps;
	if (args._deGroote) {
		auto deGroote = new DeGrooteFregly2016Muscle();
		deGroote->setName("biceps");
		deGroote->set_max_isometric_force(maxIsometricForce);
		deGroote->set_optimal_fiber_length(optimalFiberLength);
		deGroote->set_tendon_slack_length(tendonSlackLength);
		deGroote->set_pennation_angle_at_optimal(pennationAngle);
		deGroote->set_fiber_damping(args._muscleFiberDamping);
		biceps = deGroote;
		prefix = prefix + "deGroote/";
	} else {
		auto millard = new Millard2012EquilibriumMuscle(
			"biceps", maxIsometricForce, optimalFiberLength, tendonSlackLength,
			pennationAngle);
		millard->set_fiber_damping(args._muscleFiberDamping);
		biceps = millard;
		prefix = prefix + "millard/";

		std::cout << "    fiber-damping = " << millard->get_fiber_damping() << std::endl;
		if (std::isnan(args._vMax)) {
			std::cout << "    maxContractionVelocity = " << millard->getMaxContractionVelocity() << " (default)" << std::endl;
		} else {
			millard->setMaxContractionVelocity(args._vMax);
			std::cout << "    maxContractionVelocity = " << millard->getMaxContractionVelocity() << std::endl;
		}
	}

	std::cout << "Ignore tendon compliance = " << args._ignore_tendon_compliance << std::endl;
	biceps->set_ignore_tendon_compliance(args._ignore_tendon_compliance);

	biceps->addNewPathPoint("origin", *humerus, Vec3(0, 0.8, 0));
	biceps->addNewPathPoint("insertion", *radius, Vec3(0, 0.7, 0));

	// Add a controller that specifies the excitation of the muscle.
	auto brain = new PrescribedController();
	brain->setName("brain");
	brain->addActuator(*biceps);

	// Muscle excitation.
	auto ctrlfn =
		new StepFunction(args._sim_time * 0.1, args._sim_time * 0.9,
						 args._excitationInitial, args._excitationFinal);
	brain->prescribeControlForActuator("biceps", ctrlfn);

	// Add components to the model.
	model.addBody(humerus);
	model.addBody(radius);
	model.addJoint(elbow);
	model.addJoint(shoulder);
	model.addForce(biceps);
	model.addController(brain);

	// Add a console reporter to print the muscle fiber force and elbow
	// angle.The output will be written to the log file(out.log) in the current
	// directory.
	auto reporter = new TableReporter();

	reporter->setName(model.getName() + "_results");
	reporter->set_report_time_interval(args._reporter_dt);

	// Loop through the desired device outputs and add them to the reporter.
	using Pair = std::pair<std::string, std::string>;
	std::vector<Pair> rep_lst = {
		Pair("eigen_value", "eigen_value"),
		Pair("activation", "activation"),
		Pair("fiber_force", "fiberForce"),
		Pair("fiber_length", "fiberLength"),
		Pair("fiber_velocity", "fiberVelocity"),
		Pair("tendon_length", "tendonLength"),
		Pair("eigen_value", "eigen_value")};

	auto biceps_o_names = biceps->getOutputNames();
	for (auto o_name : rep_lst) {
		if (std::any_of(biceps_o_names.begin(), biceps_o_names.end(),
						[&](std::string &b) { return o_name.first == b; })) {
			reporter->addToReport(biceps->getOutput(o_name.first),
								  o_name.second);
		}
	}

	model.addComponent(reporter);

	// Add display geometry.
	auto bodyGeometry = new Ellipsoid(0.1, 0.5, 0.1);
	bodyGeometry->setColor(Vec3(0.5)); // Gray

	// Attach an ellipsoid to a frame located at the center of each body.
	auto humerusCenter = new PhysicalOffsetFrame();
	humerusCenter->setName("humerusCenter");
	humerusCenter->setParentFrame(*humerus);
	humerusCenter->setOffsetTransform(SimTK::Transform(Vec3(0, 0.5, 0)));
	humerus->addComponent(humerusCenter);
	humerusCenter->attachGeometry(bodyGeometry->clone());

	auto radiusCenter = new PhysicalOffsetFrame();
	radiusCenter->setName("radiusCenter");
	radiusCenter->setParentFrame(*radius);
	radiusCenter->setOffsetTransform(SimTK::Transform(Vec3(0, 0.5, 0)));
	radius->addComponent(radiusCenter);
	radiusCenter->attachGeometry(bodyGeometry->clone());

	// Visualize.
	if (args._visualize) {
		model.setUseVisualizer(args._visualize);
	}

	// Configure the model.
	SimTK::State &state = model.initSystem();

	shoulder->getCoordinate().setLocked(state, true);
	elbow->updCoordinate(PinJoint::Coord::RotationZ)
		.setDefaultValue(0.5 * SimTK::Pi);
	elbow->getCoordinate().setValue(state, 0.5 * SimTK::Pi);
	model.equilibrateMuscles(state);

	// Simulate.
	auto finalTime = args._sim_time;
	auto manager = new Manager(model);
	args.set_manager_args(*manager);
	/* SimTK::Integrator& integ = manager->getIntegrator(); */
	/* integ.setUseInfinityNorm(); */
	manager->initialize(state);
	manager->integrate(finalTime);

	// Print table with simulation summary.
	SimTK::Integrator integ = manager->getIntegrator();
	std::string table_delim = " | ";
	std::cout << "SimulationSummary" << table_delim << "name" << table_delim
			  << "simtime" << table_delim << "numStepsAttempted" << table_delim
			  << "numStepsTaken" << table_delim << "getNumRealizations"
			  << table_delim << "getNumIterationsstd::string path"
			  << table_delim << std::endl;

	std::cout << "SimulationSummary" << table_delim << argv[0] << "-" << prefix
			  << table_delim << finalTime << table_delim
			  << integ.getNumStepsAttempted() << table_delim
			  << integ.getNumStepsTaken() << table_delim
			  << integ.getNumRealizations() << table_delim
			  << integ.getNumIterations() << table_delim << std::endl;

	// Plot results (print to stdout).
	{
		auto table = reporter->getTable();
		auto matrix = reporter->getTable().getMatrix();
		for (int i = 0; i < matrix.nrow(); i++) {
			double time = table.getIndependentColumn()[i];
			for (int j = 0; j < matrix.ncol(); j++) {
				auto label = table.getColumnLabel(j);
				double value = matrix.row(i)[j];
				std::cout << "RUSTCSVPLOT," << prefix << label << "," << time
						  << "," << value << "," << j << std::endl;
			}
		}
	}

	{
		auto table = manager->getStatesTable();
		auto matrix = table.getMatrix();
		auto table_rep = reporter->getTable();
		auto matrix_rep = table_rep.getMatrix();
		double prev_time = 0;
		int prev_nearest_row = 0;
		int steps = 0;

		for (int i = 0; i < matrix.nrow(); i++) {
			// Log timestep.
			double time = table.getIndependentColumn()[i];
			double dt = time - prev_time;
			prev_time = time;
			std::cout << "RUSTCSVPLOT," << prefix << "dt," << time << "," << dt
					  << std::endl;
			std::cout << "RUSTCSVPLOT," << prefix << "steps," << time << ","
					  << ++steps << std::endl;

			// Log all states.
			for (int j = 0; j < matrix.ncol(); j++) {
				auto label = table.getColumnLabel(j);
				double value = matrix.row(i)[j];
				std::cout << "RUSTCSVPLOT," << prefix << label << "," << time
						  << "," << value << std::endl;
			}

			// Log reported values at state times.
			int nearest_row = table_rep.getNearestRowIndexForTime(time);
			if (nearest_row == prev_nearest_row) {
				continue;
			}
			prev_nearest_row = nearest_row;
			double time_rep = table_rep.getIndependentColumn()[nearest_row];
			for (int j = 0; j < matrix_rep.ncol(); j++) {
				auto label_rep = table_rep.getColumnLabel(j);
				double value_rep = matrix_rep.row(nearest_row)[j];
				std::cout << "RUSTCSVPLOT,nearest-" << prefix << label_rep
						  << "," << time_rep << "," << value_rep << std::endl;
			}
		}
	}

	return 0;
}
