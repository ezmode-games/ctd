// Test stubs for C++ functions - used when running cargo test
// The real implementations are in plugin.cpp and compiled via CMake

#include "ctd-fallout4/src/lib.rs.h"

namespace ctd {

rust::Vec<PluginInfo> get_load_order() {
    return rust::Vec<PluginInfo>();
}

rust::String get_game_version() {
    return rust::String("1.0.0-test");
}

rust::String get_f4se_version() {
    return rust::String("0.7.0-test");
}

}  // namespace ctd
