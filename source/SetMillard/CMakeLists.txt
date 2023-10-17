cmake_minimum_required(VERSION 3.2)

project(SetMillard)

find_package(OpenSim REQUIRED)

add_executable(SetMillard
	main.cpp
	)

set_target_properties(SetMillard PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(SetMillard PRIVATE cxx_std_17)

target_link_libraries(SetMillard
	${OpenSim_LIBRARIES}
)

install(TARGETS SetMillard
        RUNTIME DESTINATION bin
)
