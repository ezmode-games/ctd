// C++ bridge implementation stubs
// These will be overridden by the game-specific UE4SS mod

#include "ctd-ue5/src/lib.rs.h"
#include "bridge.hpp"

namespace ctd {

// Default implementations - to be overridden by game-specific code

rust::Vec<PluginInfo> get_load_order() {
    // Return empty by default - game-specific mod should override
    return rust::Vec<PluginInfo>();
}

rust::String get_game_version() {
    return rust::String("unknown");
}

}  // namespace ctd
