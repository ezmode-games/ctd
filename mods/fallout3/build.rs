//! Build script for CXX code generation.

fn main() {
    // Generate CXX bridge code
    cxx_build::bridge("src/lib.rs")
        .std("c++20")
        .include(".") // Include current dir so cpp/bridge.hpp is found
        .compile("ctd-fallout3-bridge");

    // Rerun if these files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp/bridge.hpp");
}
