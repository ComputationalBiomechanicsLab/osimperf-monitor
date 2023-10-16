cmake_minimum_required(VERSION 3.2)

project(AddVisualizer)

find_package(OpenSim REQUIRED)

add_executable(AddVisualizer
	main.cpp
	)

set_target_properties(AddVisualizer PROPERTIES
	CXX_STANDARD_REQUIRED YES
	CXX_EXTENSIONS OFF
)

target_compile_features(AddVisualizer PRIVATE cxx_std_17)

target_link_libraries(AddVisualizer
	${OpenSim_LIBRARIES}
)

install(TARGETS AddVisualizer
        RUNTIME DESTINATION bin
)
