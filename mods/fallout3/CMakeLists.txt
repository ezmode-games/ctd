cmake_minimum_required(VERSION 3.22)

# Project configuration
# NOTE: Fallout 3 is 32-bit (x86)
project(
    ctd-fallout3
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

# xFOSE (community continuation of FOSE)
FetchContent_Declare(
    xFOSE
    GIT_REPOSITORY https://github.com/AtomicTEM/xFOSE.git
    GIT_TAG main
)
FetchContent_GetProperties(xFOSE)
if(NOT xfose_POPULATED)
    FetchContent_Populate(xFOSE)
endif()

# ============================================================================
# Rust Library (Manual Build)
# ============================================================================

# Output paths for the Rust build (32-bit target)
set(RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/rust-build")
set(RUST_LIB_PATH "${RUST_TARGET_DIR}/i686-pc-windows-msvc/release/ctd_fallout3.lib")
set(CXX_BRIDGE_DIR "${RUST_TARGET_DIR}/i686-pc-windows-msvc/cxxbridge/ctd-fallout3/src")
set(CXX_BRIDGE_HEADER "${CXX_BRIDGE_DIR}/lib.rs.h")

# Custom command to build the Rust library (32-bit)
add_custom_command(
    OUTPUT ${RUST_LIB_PATH} ${CXX_BRIDGE_HEADER}
    COMMAND cargo build --release
        --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
        --target i686-pc-windows-msvc
        --target-dir "${RUST_TARGET_DIR}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust library (ctd-fallout3, 32-bit)..."
    VERBATIM
)

# Custom target that depends on Rust build outputs
add_custom_target(ctd-fallout3-build ALL
    DEPENDS ${RUST_LIB_PATH} ${CXX_BRIDGE_HEADER}
)

# Create an IMPORTED library target for the Rust static library
add_library(ctd-fallout3-lib STATIC IMPORTED GLOBAL)
set_target_properties(ctd-fallout3-lib PROPERTIES
    IMPORTED_LOCATION ${RUST_LIB_PATH}
)
add_dependencies(ctd-fallout3-lib ctd-fallout3-build)

# ============================================================================
# C++ Plugin
# ============================================================================

# Find C++ source files
file(GLOB_RECURSE CPP_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/cpp/*.cpp"
)

# Create the FOSE plugin DLL
set(PLUGIN_TARGET ctd-fallout3-plugin)
add_library(${PLUGIN_TARGET} SHARED
    ${CPP_SOURCES}
)

target_include_directories(${PLUGIN_TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/..
    # xFOSE headers
    ${xfose_SOURCE_DIR}/fose
    # CXX bridge output location
    ${RUST_TARGET_DIR}/cxxbridge
    ${RUST_TARGET_DIR}/i686-pc-windows-msvc/cxxbridge
)

target_link_libraries(${PLUGIN_TARGET} PRIVATE
    ctd-fallout3-lib
    # Windows libraries required by Rust
    DbgHelp
    ntdll
    userenv
    ws2_32
    kernel32
    bcrypt
)

# Ensure plugin waits for Rust build
add_dependencies(${PLUGIN_TARGET} ctd-fallout3-build)

# ============================================================================
# Output Configuration
# ============================================================================

set_target_properties(${PLUGIN_TARGET} PROPERTIES
    OUTPUT_NAME "ctd-fallout3"
    SUFFIX ".dll"
)

install(TARGETS ${PLUGIN_TARGET}
    RUNTIME DESTINATION "FOSE/Plugins"
)
