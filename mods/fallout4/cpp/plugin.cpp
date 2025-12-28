// F4SE Plugin Entry Point
//
// This is the C++ layer that handles F4SE registration and VEH setup.
// The actual crash processing is done in Rust.

#include <F4SE/F4SE.h>
#include <RE/Fallout.h>

#include "ctd-fallout4/src/lib.rs.h"  // CXX-generated Rust interface
#include "bridge.hpp"
#include "veh.hpp"

namespace {

void MessageHandler(F4SE::MessagingInterface::Message* message) {
    switch (message->type) {
        case F4SE::MessagingInterface::kGameDataReady:
            ctd::on_data_loaded();
            break;
        default:
            break;
    }
}

}  // namespace

// Plugin query - called by F4SE to get plugin info
extern "C" __declspec(dllexport) bool F4SEAPI F4SEPlugin_Query(
    const F4SE::QueryInterface* f4se,
    F4SE::PluginInfo* info
) {
    info->infoVersion = F4SE::PluginInfo::kVersion;
    info->name = "CTD Crash Reporter";
    info->version = 1;

    if (f4se->IsEditor()) {
        return false;
    }

    return true;
}

// Plugin load - called by F4SE after query succeeds
extern "C" __declspec(dllexport) bool F4SEAPI F4SEPlugin_Load(
    const F4SE::LoadInterface* f4se
) {
    F4SE::Init(f4se);

    // Register VEH handler for crash capture
    ctd::register_veh_handler();

    // Register for messaging events
    auto messaging = F4SE::GetMessagingInterface();
    if (messaging) {
        messaging->RegisterListener(MessageHandler);
    }

    // Initialize Rust side
    ctd::init();

    F4SE::log::info("CTD Crash Reporter loaded");
    return true;
}

namespace ctd {

// Get load order from TESDataHandler
rust::Vec<PluginInfo> get_load_order() {
    rust::Vec<PluginInfo> plugins;

    auto* handler = RE::TESDataHandler::GetSingleton();
    if (!handler) {
        return plugins;
    }

    // Iterate over loaded files
    for (auto* file : handler->files) {
        if (!file) continue;

        PluginInfo info;
        info.name = rust::String(file->filename);
        info.index = file->compileIndex;
        info.is_light = false;  // F4 doesn't have light plugins (ESL), that's Skyrim SE only
        plugins.push_back(std::move(info));
    }

    return plugins;
}

// Get game version
rust::String get_game_version() {
    // REL::Version provides the runtime version
    auto version = REL::Module::get().version();
    return rust::String(version.string());
}

// Get F4SE version
rust::String get_f4se_version() {
    return rust::String(F4SE::GetF4SEVersion().string());
}

}  // namespace ctd
