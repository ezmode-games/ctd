// FOSE Plugin Entry Point
//
// This is the C++ layer that handles FOSE registration and VEH setup.
// The actual crash processing is done in Rust.
//
// NOTE: This is scaffolding code. Full FOSE SDK integration is required
// for production use. See: https://fose.silverlock.org/

#include <Windows.h>

#include "ctd-fallout3/src/lib.rs.h"  // CXX-generated Rust interface
#include "bridge.hpp"
#include "veh.hpp"

// FOSE interface types (minimal definitions for scaffolding)
// Full definitions require FOSE SDK headers
struct FOSEInterface {
    uint32_t foseVersion;
    uint32_t runtimeVersion;
    uint32_t editorVersion;
    uint32_t isEditor;
    // ... more fields in actual SDK
};

struct PluginInfo {
    uint32_t infoVersion;
    const char* name;
    uint32_t version;
};

// Plugin query - called by FOSE to get plugin info
extern "C" __declspec(dllexport) bool FOSEPlugin_Query(
    FOSEInterface* fose,
    PluginInfo* info
) {
    info->infoVersion = 1;
    info->name = "CTD Crash Reporter";
    info->version = 1;

    // Don't load in editor
    if (fose->isEditor) {
        return false;
    }

    return true;
}

// Plugin load - called by FOSE after query succeeds
extern "C" __declspec(dllexport) bool FOSEPlugin_Load(FOSEInterface* fose) {
    // Register VEH handler for crash capture
    ctd::register_veh_handler();

    // Initialize Rust side
    ctd::init();

    return true;
}

namespace ctd {

// Get load order from game
// TODO: Implement using FOSE DataHandler access
rust::Vec<PluginInfo> get_load_order() {
    rust::Vec<PluginInfo> plugins;

    // Scaffolding: Return empty list
    // Full implementation requires FOSE SDK's DataHandler access
    // to enumerate loaded ESM/ESP files

    return plugins;
}

// Get game version
// TODO: Implement using FOSE interface
rust::String get_game_version() {
    // Scaffolding: Return placeholder
    // Full implementation would use fose->runtimeVersion
    return rust::String("1.7.0.3");  // Common FO3 version
}

// Get FOSE version
// TODO: Implement using FOSE interface
rust::String get_fose_version() {
    // Scaffolding: Return placeholder
    // Full implementation would use fose->foseVersion
    return rust::String("1.3.0");
}

}  // namespace ctd
