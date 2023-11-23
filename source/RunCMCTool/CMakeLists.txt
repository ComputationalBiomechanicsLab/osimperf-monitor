cmake_minimum_required(VERSION 3.2)

project(RunCMCTool)

find_package(OpenSim REQUIRED)

add_executable(RunCMCTool
	main.cpp
	)

set_target_properties(RunCMCTool PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(RunCMCTool PRIVATE cxx_std_17)

target_link_libraries(RunCMCTool
	${OpenSim_LIBRARIES}
)

install(TARGETS RunCMCTool
        RUNTIME DESTINATION bin
)
