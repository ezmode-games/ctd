#pragma once

#include <string>
#include <vector>
#include "rust/cxx.h"

namespace ctd {

/// Plugin info for load order (matches Rust struct)
struct PluginInfo;

/// Get the load order from the game
/// This is implemented by the game-specific UE4SS mod
rust::Vec<PluginInfo> get_load_order();

/// Get game version string
/// This is implemented by the game-specific UE4SS mod
rust::String get_game_version();

}  // namespace ctd
