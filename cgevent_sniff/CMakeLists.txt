cmake_minimum_required(VERSION 3.20)

# Project name and version
project(cgevent_sniff VERSION 1.0.0 LANGUAGES C)

# Set C standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# macOS-specific settings
set(CMAKE_OSX_DEPLOYMENT_TARGET "10.15" CACHE STRING "Minimum macOS deployment target")

# Ensure we're on macOS
if(NOT APPLE)
    message(FATAL_ERROR "This project is macOS-only")
endif()

# Find required frameworks
find_library(APPLICATION_SERVICES_FRAMEWORK ApplicationServices REQUIRED)
find_library(CORE_FOUNDATION_FRAMEWORK CoreFoundation REQUIRED)

# Common compile options
add_compile_options(
    -Wall
    -Wextra
    -Wpedantic
    -Werror=implicit-function-declaration
)

# Common link options
# add_link_options(
#     -framework ApplicationServices
#     -framework CoreFoundation
# )

# Executable: cgevent_sniff
add_executable(cgevent_sniff cgevent_sniff.c)
target_link_libraries(cgevent_sniff
    ${APPLICATION_SERVICES_FRAMEWORK}
    ${CORE_FOUNDATION_FRAMEWORK}
)

# Set output directories
set_target_properties(cgevent_sniff
    PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin
)

# Install rules
install(TARGETS cgevent_sniff
    RUNTIME DESTINATION bin
)

# Add a custom target for building both executables
add_custom_target(all-executables
    DEPENDS cgevent_sniff
    COMMENT "Building all executables"
)

# Print configuration info
message(STATUS "Project: ${PROJECT_NAME} v${PROJECT_VERSION}")
message(STATUS "C Standard: ${CMAKE_C_STANDARD}")
message(STATUS "macOS Deployment Target: ${CMAKE_OSX_DEPLOYMENT_TARGET}")
message(STATUS "Build Type: ${CMAKE_BUILD_TYPE}") 
