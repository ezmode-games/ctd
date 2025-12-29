cmake_minimum_required(VERSION 3.22)

project(CTDCrashReporter)

# ============================================================================
# UE4SS Dependencies
# ============================================================================
# Two modes:
# 1. Local build: uses RE-UE4SS submodule (default)
# 2. CI/Pre-built: set UE4SS_PREBUILT_DIR to downloaded deps

if(DEFINED UE4SS_PREBUILT_DIR)
    message(STATUS "Using pre-built UE4SS from: ${UE4SS_PREBUILT_DIR}")

    # Create imported library for UE4SS
    add_library(UE4SS STATIC IMPORTED GLOBAL)
    set_target_properties(UE4SS PROPERTIES
        IMPORTED_LOCATION "${UE4SS_PREBUILT_DIR}/lib/UE4SS.lib"
    )

    # Link all the dependency libs
    file(GLOB UE4SS_DEP_LIBS "${UE4SS_PREBUILT_DIR}/lib/*.lib")

    # Set include directories
    target_include_directories(UE4SS INTERFACE
        "${UE4SS_PREBUILT_DIR}/include"
        "${UE4SS_PREBUILT_DIR}/include/UE4SS"
        "${UE4SS_PREBUILT_DIR}/include/Unreal"
    )
else()
    # Local build: use git submodule
    add_subdirectory(RE-UE4SS)
endif()

# ============================================================================
# Rust Library (ctd-ue5)
# ============================================================================

# Output paths for the Rust build
set(RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/rust-build")
set(RUST_LIB_PATH "${RUST_TARGET_DIR}/x86_64-pc-windows-msvc/release/ctd_ue5.lib")
set(CXX_BRIDGE_DIR "${RUST_TARGET_DIR}/cxxbridge")
set(CXX_BRIDGE_HEADER "${CXX_BRIDGE_DIR}/ctd-ue5/src/lib.rs.h")

# UE5 mod Cargo.toml is in ../ue5
set(UE5_MOD_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../ue5")

# Custom command to build the Rust library
add_custom_command(
    OUTPUT ${RUST_LIB_PATH} ${CXX_BRIDGE_HEADER}
    COMMAND cargo build --release
        --manifest-path "${UE5_MOD_DIR}/Cargo.toml"
        --target x86_64-pc-windows-msvc
        --target-dir "${RUST_TARGET_DIR}"
    WORKING_DIRECTORY ${UE5_MOD_DIR}
    COMMENT "Building Rust library (ctd-ue5)..."
    VERBATIM
)

add_custom_target(ctd-ue5-build ALL
    DEPENDS ${RUST_LIB_PATH} ${CXX_BRIDGE_HEADER}
)

add_library(ctd-ue5-lib STATIC IMPORTED GLOBAL)
set_target_properties(ctd-ue5-lib PROPERTIES
    IMPORTED_LOCATION ${RUST_LIB_PATH}
)
add_dependencies(ctd-ue5-lib ctd-ue5-build)

# ============================================================================
# UE4SS C++ Mod
# ============================================================================

set(MOD_TARGET CTDCrashReporter)

add_library(${MOD_TARGET} SHARED
    cpp/dllmain.cpp
)

target_include_directories(${MOD_TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CXX_BRIDGE_DIR}
    ${UE5_MOD_DIR}/cpp
    $<$<BOOL:${UE4SS_PREBUILT_DIR}>:${UE4SS_PREBUILT_DIR}/include>
    $<$<BOOL:${UE4SS_PREBUILT_DIR}>:${UE4SS_PREBUILT_DIR}/include/UE4SS>
    $<$<BOOL:${UE4SS_PREBUILT_DIR}>:${UE4SS_PREBUILT_DIR}/include/Unreal>
)

target_link_libraries(${MOD_TARGET} PRIVATE
    UE4SS
    ctd-ue5-lib
    DbgHelp
    ntdll
    userenv
    ws2_32
    kernel32
    bcrypt
    runtimeobject
)

# Link all pre-built UE4SS dependency libs
if(DEFINED UE4SS_PREBUILT_DIR)
    target_link_libraries(${MOD_TARGET} PRIVATE ${UE4SS_DEP_LIBS})
endif()

add_dependencies(${MOD_TARGET} ctd-ue5-build)

# Output as main.dll in UE4SS mod structure
set_target_properties(${MOD_TARGET} PROPERTIES
    OUTPUT_NAME "main"
    SUFFIX ".dll"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/CTDCrashReporter/dlls"
)
