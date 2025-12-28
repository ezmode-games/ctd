fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("cpp/bridge.cpp")
        .std("c++20")
        .compile("ctd-ue5-cxx");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp/bridge.hpp");
    println!("cargo:rerun-if-changed=cpp/bridge.cpp");
}
