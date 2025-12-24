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
# Use OVERRIDE_FIND_PACKAGE to satisfy find_package(fmt) calls
FetchContent_Declare(
    fmt
    GIT_REPOSITORY https://github.com/fmtlib/fmt.git
    GIT_TAG 10.2.1
    OVERRIDE_FIND_PACKAGE
)
FetchContent_MakeAvailable(fmt)

# spdlog (required by CommonLibSSE-NG)
FetchContent_Declare(
    spdlog
    GIT_REPOSITORY https://github.com/gabime/spdlog.git
    GIT_TAG v1.14.1
    OVERRIDE_FIND_PACKAGE
)
set(SPDLOG_FMT_EXTERNAL ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(spdlog)

# rapidcsv (required by CommonLibSSE-NG)
FetchContent_Declare(
    rapidcsv
    GIT_REPOSITORY https://github.com/d99kris/rapidcsv.git
    GIT_TAG v8.83
)
FetchContent_MakeAvailable(rapidcsv)

# CommonLibSSE-NG
# Manually populate and add with EXCLUDE_FROM_ALL + SYSTEM to avoid export issues
FetchContent_Declare(
    CommonLibSSE
    GIT_REPOSITORY https://github.com/CharmedBaryon/CommonLibSSE-NG.git
    GIT_TAG v3.6.0
)
set(BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(ENABLE_SKYRIM_SE ON CACHE BOOL "" FORCE)
set(ENABLE_SKYRIM_AE ON CACHE BOOL "" FORCE)
set(ENABLE_SKYRIM_VR OFF CACHE BOOL "" FORCE)
set(RAPIDCSV_INCLUDE_DIRS ${rapidcsv_SOURCE_DIR}/src CACHE PATH "" FORCE)

# Skip install rules for CommonLibSSE to avoid export conflicts
FetchContent_GetProperties(CommonLibSSE)
if(NOT commonlibsse_POPULATED)
    FetchContent_Populate(CommonLibSSE)
    # Disable install() commands in the subdirectory
    set(CMAKE_SKIP_INSTALL_RULES ON)
    add_subdirectory(${commonlibsse_SOURCE_DIR} ${commonlibsse_BINARY_DIR} EXCLUDE_FROM_ALL)
    set(CMAKE_SKIP_INSTALL_RULES OFF)
endif()

# ============================================================================
# Rust Library
# ============================================================================

# Import the Rust static library
corrosion_import_crate(MANIFEST_PATH Cargo.toml)

# ============================================================================
# C++ Plugin
# ============================================================================

# Find C++ source files
file(GLOB_RECURSE CPP_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/cpp/*.cpp"
)

# Create the SKSE plugin DLL (use different name to avoid collision with Rust lib)
set(PLUGIN_TARGET ctd-skyrim-plugin)
add_library(${PLUGIN_TARGET} SHARED
    ${CPP_SOURCES}
)

target_include_directories(${PLUGIN_TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    # CXX bridge output locations
    ${CMAKE_BINARY_DIR}/cargo/build/x86_64-pc-windows-msvc/cxxbridge
    ${CMAKE_CURRENT_BINARY_DIR}/corrosion_generated/cxxbridge/ctd-skyrim
)

target_link_libraries(${PLUGIN_TARGET} PRIVATE
    CommonLibSSE
    ctd-skyrim
    DbgHelp
)

# Precompiled headers (optional but speeds up builds)
target_precompile_headers(${PLUGIN_TARGET} PRIVATE
    <RE/Skyrim.h>
    <SKSE/SKSE.h>
)

# ============================================================================
# Output Configuration
# ============================================================================

# Output to a convenient location
set_target_properties(${PLUGIN_TARGET} PROPERTIES
    OUTPUT_NAME "ctd-skyrim"
    SUFFIX ".dll"
)

# Install target for distribution
install(TARGETS ${PLUGIN_TARGET}
    RUNTIME DESTINATION "SKSE/Plugins"
)
