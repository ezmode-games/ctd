// Test stubs for C++ functions - used when running cargo test
// The real implementations are in plugin.cpp and compiled via CMake

#include "ctd-skyrim/src/lib.rs.h"

namespace ctd {

rust::Vec<ModInfo> get_load_order() {
    return rust::Vec<ModInfo>();
}

rust::String get_game_version() {
    return rust::String("1.0.0-test");
}

rust::String get_skse_version() {
    return rust::String("2.0.0-test");
}

}  // namespace ctd
