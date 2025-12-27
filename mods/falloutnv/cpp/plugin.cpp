// NVSE Plugin Entry Point
//
// This is the C++ layer that handles NVSE registration and VEH setup.
// The actual crash processing is done in Rust.

// Standard library headers (required by xNVSE headers)
#include <cstdint>
#include <string>
#include <vector>
#include <unordered_map>

// xNVSE common types (must come before other NVSE headers)
#include "common/ITypes.h"
#include "nvse/nvse_version.h"

#include "PluginAPI.h"
#include "ctd-falloutnv/src/lib.rs.h"  // CXX-generated Rust interface
#include "bridge.hpp"
#include "veh.hpp"

// Global NVSE interface pointers
static NVSEInterface* g_nvse = nullptr;
static NVSEMessagingInterface* g_messaging = nullptr;

// Plugin version info
static constexpr uint32_t kPluginVersion = 1;

// Message handler
void MessageHandler(NVSEMessagingInterface::Message* msg) {
    switch (msg->type) {
        case NVSEMessagingInterface::kMessage_PostLoadGame:
        case NVSEMessagingInterface::kMessage_PostLoad:
            ctd::on_data_loaded();
            break;
        default:
            break;
    }
}

extern "C" {

// Called by NVSE to query plugin info
__declspec(dllexport) bool NVSEPlugin_Query(const NVSEInterface* nvse, PluginInfo* info) {
    info->infoVersion = PluginInfo::kInfoVersion;
    info->name = "CTD Crash Reporter";
    info->version = kPluginVersion;

    // Check NVSE version
    if (nvse->nvseVersion < NVSE_VERSION_INTEGER) {
        return false;
    }

    // Don't load in editor
    if (nvse->isEditor) {
        return false;
    }

    return true;
}

// Called by NVSE to load the plugin
__declspec(dllexport) bool NVSEPlugin_Load(NVSEInterface* nvse) {
    g_nvse = nvse;

    // Register VEH handler for crash capture
    ctd::register_veh_handler();

    // Get messaging interface
    g_messaging = static_cast<NVSEMessagingInterface*>(
        nvse->QueryInterface(kInterface_Messaging)
    );

    if (g_messaging) {
        g_messaging->RegisterListener(nvse->GetPluginHandle(), "NVSE", MessageHandler);
    }

    // Initialize Rust side
    ctd::init();

    return true;
}

}  // extern "C"

namespace ctd {

// Get load order from the game's data handler
rust::Vec<PluginInfo> get_load_order() {
    rust::Vec<PluginInfo> plugins;

    // Access TESDataHandler to get loaded plugins
    // Note: This uses NVSE's reverse-engineered game structures
    // The actual implementation depends on NVSE's DataHandler access

    // For now, return empty - will need NVSE-specific implementation
    // to access DataHandler::Get()->loadedMods

    return plugins;
}

// Get game version
rust::String get_game_version() {
    if (g_nvse) {
        // NVSE provides game version info
        uint32_t version = g_nvse->runtimeVersion;
        char buf[32];
        snprintf(buf, sizeof(buf), "%d.%d.%d.%d",
            (version >> 24) & 0xFF,
            (version >> 16) & 0xFF,
            (version >> 8) & 0xFF,
            version & 0xFF);
        return rust::String(buf);
    }
    return rust::String("unknown");
}

// Get NVSE version
rust::String get_nvse_version() {
    if (g_nvse) {
        uint32_t version = g_nvse->nvseVersion;
        char buf[32];
        snprintf(buf, sizeof(buf), "%d.%d.%d",
            (version >> 24) & 0xFF,
            (version >> 16) & 0xFF,
            version & 0xFF);
        return rust::String(buf);
    }
    return rust::String("unknown");
}

}  // namespace ctd
