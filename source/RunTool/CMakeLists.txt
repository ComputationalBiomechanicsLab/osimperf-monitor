cmake_minimum_required(VERSION 3.2)

project(RunTool)

find_package(OpenSim REQUIRED)

add_executable(RunTool
	main.cpp
	)

set_target_properties(RunTool PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(RunTool PRIVATE cxx_std_17)

target_link_libraries(RunTool
	${OpenSim_LIBRARIES}
)

install(TARGETS RunTool
        RUNTIME DESTINATION bin
)
