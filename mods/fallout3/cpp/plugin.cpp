// FOSE Plugin Entry Point
//
// This is the C++ layer that handles FOSE registration and VEH setup.
// The actual crash processing is done in Rust.

#include <cstdint>

// xFOSE common types (must come before other FOSE headers)
#include "common/ITypes.h"

#include "fose/PluginAPI.h"
#include "ctd-fallout3/src/lib.rs.h"  // CXX-generated Rust interface
#include "bridge.hpp"
#include "veh.hpp"

// Global FOSE interface pointers
static FOSEInterface* g_fose = nullptr;
static FOSEMessagingInterface* g_messaging = nullptr;

// Plugin version info
static constexpr uint32_t kPluginVersion = 1;

// Message handler
void MessageHandler(FOSEMessagingInterface::Message* msg) {
    switch (msg->type) {
        case FOSEMessagingInterface::kMessage_PostLoadGame:
        case FOSEMessagingInterface::kMessage_PostLoad:
            ctd::on_data_loaded();
            break;
        default:
            break;
    }
}

extern "C" {

// Called by FOSE to query plugin info
__declspec(dllexport) bool FOSEPlugin_Query(const FOSEInterface* fose, PluginInfo* info) {
    info->infoVersion = PluginInfo::kInfoVersion;
    info->name = "CTD Crash Reporter";
    info->version = kPluginVersion;

    // Check FOSE version
    if (fose->foseVersion < FOSE_VERSION_INTEGER) {
        return false;
    }

    // Don't load in editor
    if (fose->isEditor) {
        return false;
    }

    return true;
}

// Called by FOSE to load the plugin
__declspec(dllexport) bool FOSEPlugin_Load(FOSEInterface* fose) {
    g_fose = fose;

    // Register VEH handler for crash capture
    ctd::register_veh_handler();

    // Get messaging interface
    g_messaging = static_cast<FOSEMessagingInterface*>(
        fose->QueryInterface(kInterface_Messaging)
    );

    if (g_messaging) {
        g_messaging->RegisterListener(fose->GetPluginHandle(), "FOSE", MessageHandler);
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

    // Access DataHandler to get loaded plugins
    // Note: This uses FOSE's reverse-engineered game structures
    // The actual implementation depends on FOSE's DataHandler access

    // For now, return empty - will need FOSE-specific implementation
    // to access DataHandler::Get()->loadedMods

    return plugins;
}

// Get game version
rust::String get_game_version() {
    if (g_fose) {
        // FOSE provides game version info
        uint32_t version = g_fose->runtimeVersion;
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

// Get FOSE version
rust::String get_fose_version() {
    if (g_fose) {
        uint32_t version = g_fose->foseVersion;
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
