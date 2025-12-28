//! Build script for CXX code generation.

fn main() {
    // Generate CXX bridge code
    let mut build = cxx_build::bridge("src/lib.rs");
    build.std("c++20").include("."); // Include current dir so cpp/bridge.hpp is found

    // Include test stubs when building for cargo (not CMake)
    // The real implementations come from plugin.cpp compiled via CMake
    build.file("cpp/stubs.cpp");

    build.compile("ctd-skyrim-bridge");

    // Rerun if these files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp/bridge.hpp");
    println!("cargo:rerun-if-changed=cpp/stubs.cpp");
}
