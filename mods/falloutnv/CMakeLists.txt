cmake_minimum_required(VERSION 3.22)

# Project configuration
# NOTE: Fallout New Vegas is 32-bit (x86)
project(
    ctd-falloutnv
    VERSION 0.1.0
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION ON)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_DEBUG OFF)

# MSVC runtime library (must match Rust)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")

# ============================================================================
# Dependencies
# ============================================================================

include(FetchContent)

# xNVSE (community continuation of NVSE)
FetchContent_Declare(
    xNVSE
    GIT_REPOSITORY https://github.com/xNVSE/NVSE.git
    GIT_TAG main
)
FetchContent_GetProperties(xNVSE)
if(NOT xnvse_POPULATED)
    FetchContent_Populate(xNVSE)
endif()

# ============================================================================
# Rust Library (Manual Build)
# ============================================================================

# Output paths for the Rust build (32-bit target)
set(RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/rust-build")
set(RUST_LIB_PATH "${RUST_TARGET_DIR}/i686-pc-windows-msvc/release/ctd_falloutnv.lib")
set(CXX_BRIDGE_DIR "${RUST_TARGET_DIR}/i686-pc-windows-msvc/cxxbridge/ctd-falloutnv/src")
set(CXX_BRIDGE_HEADER "${CXX_BRIDGE_DIR}/lib.rs.h")

# Custom command to build the Rust library (32-bit)
add_custom_command(
    OUTPUT ${RUST_LIB_PATH} ${CXX_BRIDGE_HEADER}
    COMMAND cargo build --release
        --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
        --target i686-pc-windows-msvc
        --target-dir "${RUST_TARGET_DIR}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust library (ctd-falloutnv, 32-bit)..."
    VERBATIM
)

# Custom target that depends on Rust build outputs
add_custom_target(ctd-falloutnv-build ALL
    DEPENDS ${RUST_LIB_PATH} ${CXX_BRIDGE_HEADER}
)

# Create an IMPORTED library target for the Rust static library
add_library(ctd-falloutnv-lib STATIC IMPORTED GLOBAL)
set_target_properties(ctd-falloutnv-lib PROPERTIES
    IMPORTED_LOCATION ${RUST_LIB_PATH}
)
add_dependencies(ctd-falloutnv-lib ctd-falloutnv-build)

# ============================================================================
# C++ Plugin
# ============================================================================

# Find C++ source files
file(GLOB_RECURSE CPP_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/cpp/*.cpp"
)

# Create the NVSE plugin DLL
set(PLUGIN_TARGET ctd-falloutnv-plugin)
add_library(${PLUGIN_TARGET} SHARED
    ${CPP_SOURCES}
)

target_include_directories(${PLUGIN_TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/..
    # xNVSE headers
    ${xnvse_SOURCE_DIR}/nvse
    ${xnvse_SOURCE_DIR}/nvse/nvse
    # CXX bridge output location
    ${RUST_TARGET_DIR}/cxxbridge
    ${RUST_TARGET_DIR}/i686-pc-windows-msvc/cxxbridge
)

target_link_libraries(${PLUGIN_TARGET} PRIVATE
    ctd-falloutnv-lib
    # Windows libraries required by Rust
    DbgHelp
    ntdll
    userenv
    ws2_32
    kernel32
    bcrypt
)

# Ensure plugin waits for Rust build
add_dependencies(${PLUGIN_TARGET} ctd-falloutnv-build)

# ============================================================================
# Output Configuration
# ============================================================================

set_target_properties(${PLUGIN_TARGET} PROPERTIES
    OUTPUT_NAME "ctd-falloutnv"
    SUFFIX ".dll"
)

install(TARGETS ${PLUGIN_TARGET}
    RUNTIME DESTINATION "NVSE/Plugins"
)
