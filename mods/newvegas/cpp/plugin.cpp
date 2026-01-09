// NVSE Plugin Entry Point
//
// This is the C++ layer that handles NVSE registration and VEH setup.
// The actual crash processing is done in Rust.
//
// NOTE: This is scaffolding code. Full NVSE/xNVSE SDK integration is required
// for production use. See: https://github.com/xNVSE/NVSE

#include <Windows.h>

#include "ctd-newvegas/src/lib.rs.h"  // CXX-generated Rust interface
#include "bridge.hpp"
#include "veh.hpp"

// NVSE interface types (minimal definitions for scaffolding)
// Full definitions require NVSE SDK headers
struct NVSEInterface {
    uint32_t nvseVersion;
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

// Plugin query - called by NVSE to get plugin info
extern "C" __declspec(dllexport) bool NVSEPlugin_Query(
    NVSEInterface* nvse,
    PluginInfo* info
) {
    info->infoVersion = 1;
    info->name = "CTD Crash Reporter";
    info->version = 1;

    // Don't load in editor
    if (nvse->isEditor) {
        return false;
    }

    return true;
}

// Plugin load - called by NVSE after query succeeds
extern "C" __declspec(dllexport) bool NVSEPlugin_Load(NVSEInterface* nvse) {
    // Register VEH handler for crash capture
    ctd::register_veh_handler();

    // Initialize Rust side
    ctd::init();

    return true;
}

namespace ctd {

// Get load order from game
// TODO: Implement using NVSE DataHandler access
rust::Vec<PluginInfo> get_load_order() {
    rust::Vec<PluginInfo> plugins;

    // Scaffolding: Return empty list
    // Full implementation requires NVSE SDK's DataHandler access
    // to enumerate loaded ESM/ESP files

    return plugins;
}

// Get game version
// TODO: Implement using NVSE interface
rust::String get_game_version() {
    // Scaffolding: Return placeholder
    // Full implementation would use nvse->runtimeVersion
    return rust::String("1.4.0.525");  // Common FNV version
}

// Get NVSE version
// TODO: Implement using NVSE interface
rust::String get_nvse_version() {
    // Scaffolding: Return placeholder
    // Full implementation would use nvse->nvseVersion
    return rust::String("6.3.0");  // xNVSE version
}

}  // namespace ctd
