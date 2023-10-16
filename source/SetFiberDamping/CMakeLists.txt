cmake_minimum_required(VERSION 3.2)

project(SetFiberDamping)

find_package(OpenSim REQUIRED)

add_executable(SetFiberDamping
	main.cpp
	)

set_target_properties(SetFiberDamping PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(SetFiberDamping PRIVATE cxx_std_17)

target_link_libraries(SetFiberDamping
	${OpenSim_LIBRARIES}
)

install(TARGETS SetFiberDamping
        RUNTIME DESTINATION bin
)
