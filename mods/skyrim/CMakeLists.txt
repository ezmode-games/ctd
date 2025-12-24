cmake_minimum_required(VERSION 3.22)

# Project configuration
project(
    ctd-skyrim
    VERSION 0.1.0
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION ON)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_DEBUG OFF)

# MSVC runtime library (must match Rust)
# Use dynamically linked runtime (DLL) to match Rust default
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")

# ============================================================================
# Dependencies
# ============================================================================

include(FetchContent)

# Corrosion for Rust/CMake integration
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5
)
FetchContent_MakeAvailable(Corrosion)

# fmt (required by CommonLibSSE-NG)
FetchContent_Declare(
    fmt
    GIT_REPOSITORY https://github.com/fmtlib/fmt.git
    GIT_TAG 10.2.1
)
FetchContent_MakeAvailable(fmt)

# spdlog (required by CommonLibSSE-NG)
FetchContent_Declare(
    spdlog
    GIT_REPOSITORY https://github.com/gabime/spdlog.git
    GIT_TAG v1.14.1
)
set(SPDLOG_FMT_EXTERNAL ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(spdlog)

# CommonLibSSE-NG
FetchContent_Declare(
    CommonLibSSE
    GIT_REPOSITORY https://github.com/CharmedBaryon/CommonLibSSE-NG.git
    GIT_TAG v3.6.0
)
set(BUILD_TESTS OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(CommonLibSSE)

# ============================================================================
# Rust Library
# ============================================================================

# Import the Rust static library
corrosion_import_crate(MANIFEST_PATH Cargo.toml)

# Set Rust flags
corrosion_set_env_vars(ctd-skyrim
    "CXXBRIDGE_DIR=${CMAKE_CURRENT_BINARY_DIR}/cxxbridge"
)

# ============================================================================
# C++ Plugin
# ============================================================================

# Find C++ source files
file(GLOB_RECURSE CPP_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/cpp/*.cpp"
)

# Create the SKSE plugin DLL
add_library(${PROJECT_NAME} SHARED
    ${CPP_SOURCES}
)

target_include_directories(${PROJECT_NAME} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge
    ${CMAKE_CURRENT_BINARY_DIR}/cargo/build/x86_64-pc-windows-msvc/cxxbridge
)

target_link_libraries(${PROJECT_NAME} PRIVATE
    CommonLibSSE
    ctd-skyrim
    DbgHelp
)

# Precompiled headers (optional but speeds up builds)
target_precompile_headers(${PROJECT_NAME} PRIVATE
    <RE/Skyrim.h>
    <SKSE/SKSE.h>
)

# ============================================================================
# Output Configuration
# ============================================================================

# Output to a convenient location
set_target_properties(${PROJECT_NAME} PROPERTIES
    OUTPUT_NAME "ctd-skyrim"
    SUFFIX ".dll"
)

# Install target for distribution
install(TARGETS ${PROJECT_NAME}
    RUNTIME DESTINATION "SKSE/Plugins"
)
