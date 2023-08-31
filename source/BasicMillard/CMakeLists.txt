cmake_minimum_required(VERSION 3.2)

project(BasicMillard)

find_package(OpenSim REQUIRED)

add_executable(BasicMillard
	main.cpp
	)

set_target_properties(BasicMillard PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(BasicMillard PRIVATE cxx_std_17)

target_link_libraries(BasicMillard
	${OpenSim_LIBRARIES}
)

install(TARGETS BasicMillard
        RUNTIME DESTINATION bin
)
