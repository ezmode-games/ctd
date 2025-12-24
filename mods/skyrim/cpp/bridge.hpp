#pragma once

// This header is included by both CXX-generated code and our C++ implementation.
// It declares the C++ functions that Rust can call.

#include <cstdint>
#include <string>
#include <vector>

#include "rust/cxx.h"

namespace ctd {

// Forward declare the Rust types (defined in lib.rs.h)
struct ModInfo;

// C++ functions callable from Rust
rust::Vec<ModInfo> get_load_order();
rust::String get_game_version();
rust::String get_skse_version();

} // namespace ctd
