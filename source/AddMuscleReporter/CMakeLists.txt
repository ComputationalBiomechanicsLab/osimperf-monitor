cmake_minimum_required(VERSION 3.2)

project(AddMuscleReporter)

find_package(OpenSim REQUIRED)

add_executable(AddMuscleReporter
	main.cpp
	)

set_target_properties(AddMuscleReporter PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(AddMuscleReporter PRIVATE cxx_std_17)

target_link_libraries(AddMuscleReporter
	${OpenSim_LIBRARIES}
)

install(TARGETS AddMuscleReporter
        RUNTIME DESTINATION bin
)
