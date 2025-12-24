// SKSE64 Plugin Entry Point
//
// This is the C++ layer that handles SKSE registration and VEH setup.
// The actual crash processing is done in Rust.

#include <SKSE/SKSE.h>
#include <RE/Skyrim.h>

#include "ctd-skyrim/src/lib.rs.h"  // CXX-generated Rust interface
#include "bridge.hpp"
#include "veh.hpp"

namespace {

    void MessageHandler(SKSE::MessagingInterface::Message* message) {
        switch (message->type) {
            case SKSE::MessagingInterface::kDataLoaded:
                ctd::on_data_loaded();
                break;
            default:
                break;
        }
    }

}  // namespace

// Plugin query - called by SKSE to get plugin info
extern "C" __declspec(dllexport) bool SKSEAPI SKSEPlugin_Query(
    const SKSE::QueryInterface* skse,
    SKSE::PluginInfo* info
) {
    info->infoVersion = SKSE::PluginInfo::kVersion;
    info->name = "CTD Crash Reporter";
    info->version = 1;

    if (skse->IsEditor()) {
        return false;
    }

    return true;
}

// Plugin load - called by SKSE after query succeeds
extern "C" __declspec(dllexport) bool SKSEAPI SKSEPlugin_Load(
    const SKSE::LoadInterface* skse
) {
    SKSE::Init(skse);

    // Register VEH handler for crash capture
    ctd::register_veh_handler();

    // Register for messaging events
    auto messaging = SKSE::GetMessagingInterface();
    if (messaging) {
        messaging->RegisterListener(MessageHandler);
    }

    // Initialize Rust side
    ctd::init();

    SKSE::log::info("CTD Crash Reporter loaded");
    return true;
}

namespace ctd {

// Get load order from TESDataHandler
rust::Vec<ModInfo> get_load_order() {
    rust::Vec<ModInfo> mods;

    auto* handler = RE::TESDataHandler::GetSingleton();
    if (!handler) {
        return mods;
    }

    for (auto* file : handler->files) {
        if (!file) continue;

        ModInfo info;
        info.name = rust::String(file->fileName);
        info.index = file->compileIndex;
        info.is_light = file->IsLight();
        mods.push_back(std::move(info));
    }

    return mods;
}

// Get game version
rust::String get_game_version() {
    // REL::Version provides the runtime version
    auto version = REL::Module::get().version();
    return rust::String(version.string());
}

// Get SKSE version
rust::String get_skse_version() {
    return rust::String(SKSE::PluginDeclaration::GetSingleton()->GetVersion().string());
}

}  // namespace ctd
