cmake_minimum_required(VERSION 3.2)

project(Hopper)

find_package(OpenSim REQUIRED)

add_executable(Hopper
    exampleHopperDevice.cpp
    buildHopperModel.cpp
    buildDeviceModel.cpp
    buildTestbedModel.cpp
    defineDeviceAndController.h)

set_target_properties(Hopper PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(Hopper PRIVATE cxx_std_11)

target_link_libraries(Hopper
	${OpenSim_LIBRARIES}
    # ${Simbody_LIBRARIES}
)

# Set the installation prefix to a custom directory
# set(CMAKE_INSTALL_PREFIX /path/to/installation/directory)

# Specify the installation paths for binaries and headers
install(TARGETS Hopper
        RUNTIME DESTINATION bin
)
