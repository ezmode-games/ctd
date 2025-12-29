// CTD Crash Reporter - Oblivion Remastered UE4SS Mod
// This is a UE4SS C++ mod that wraps the ctd-ue5 Rust library

#include <Mod/CppUserModBase.hpp>
#include <DynamicOutput/DynamicOutput.hpp>
#include <Unreal/UObjectGlobals.hpp>
#include <Unreal/UnrealVersion.hpp>

#include <filesystem>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

// Windows headers for version info
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
#pragma comment(lib, "version.lib")

// Include the CXX bridge header from ctd-ue5
#include "ctd-ue5/src/lib.rs.h"

using namespace RC;
using namespace RC::Unreal;

// ============================================================================
// Helper functions
// ============================================================================

namespace {

// Get the game executable path
std::filesystem::path get_game_exe_path()
{
    wchar_t path[MAX_PATH];
    GetModuleFileNameW(nullptr, path, MAX_PATH);
    return std::filesystem::path(path);
}

// Get file version from a Windows executable
std::string get_file_version(const std::filesystem::path& exe_path)
{
    DWORD handle = 0;
    DWORD size = GetFileVersionInfoSizeW(exe_path.c_str(), &handle);
    if (size == 0)
    {
        return "unknown";
    }

    std::vector<BYTE> data(size);
    if (!GetFileVersionInfoW(exe_path.c_str(), handle, size, data.data()))
    {
        return "unknown";
    }

    VS_FIXEDFILEINFO* file_info = nullptr;
    UINT len = 0;
    if (!VerQueryValueW(data.data(), L"\\", reinterpret_cast<void**>(&file_info), &len))
    {
        return "unknown";
    }

    if (file_info == nullptr)
    {
        return "unknown";
    }

    std::ostringstream version;
    version << HIWORD(file_info->dwFileVersionMS) << "."
            << LOWORD(file_info->dwFileVersionMS) << "."
            << HIWORD(file_info->dwFileVersionLS) << "."
            << LOWORD(file_info->dwFileVersionLS);

    return version.str();
}

// Get UE4SS mods directory
std::filesystem::path get_mods_directory()
{
    auto exe_path = get_game_exe_path();
    auto game_dir = exe_path.parent_path();

    // UE4SS mods are typically in: <GameDir>/ue4ss/Mods or <GameDir>/Mods
    auto ue4ss_mods = game_dir / "ue4ss" / "Mods";
    if (std::filesystem::exists(ue4ss_mods))
    {
        return ue4ss_mods;
    }

    auto direct_mods = game_dir / "Mods";
    if (std::filesystem::exists(direct_mods))
    {
        return direct_mods;
    }

    return {};
}

// Parse mods.txt to get enabled mods
std::vector<std::pair<std::string, bool>> parse_mods_txt(const std::filesystem::path& mods_txt_path)
{
    std::vector<std::pair<std::string, bool>> mods;

    std::ifstream file(mods_txt_path);
    if (!file.is_open())
    {
        return mods;
    }

    std::string line;
    while (std::getline(file, line))
    {
        // Skip empty lines and comments
        if (line.empty() || line[0] == ';' || line[0] == '#')
        {
            continue;
        }

        // Parse "ModName : 1" or "ModName : 0" format
        auto colon_pos = line.find(':');
        if (colon_pos != std::string::npos)
        {
            std::string mod_name = line.substr(0, colon_pos);
            std::string enabled_str = line.substr(colon_pos + 1);

            // Trim whitespace
            while (!mod_name.empty() && (mod_name.back() == ' ' || mod_name.back() == '\t'))
            {
                mod_name.pop_back();
            }
            while (!enabled_str.empty() && (enabled_str.front() == ' ' || enabled_str.front() == '\t'))
            {
                enabled_str.erase(0, 1);
            }

            bool enabled = !enabled_str.empty() && enabled_str[0] == '1';
            mods.emplace_back(mod_name, enabled);
        }
    }

    return mods;
}

// Scan Mods directory for installed mods
std::vector<std::string> scan_mods_directory(const std::filesystem::path& mods_dir)
{
    std::vector<std::string> mods;

    if (!std::filesystem::exists(mods_dir))
    {
        return mods;
    }

    for (const auto& entry : std::filesystem::directory_iterator(mods_dir))
    {
        if (entry.is_directory())
        {
            std::string mod_name = entry.path().filename().string();
            // Skip shared and internal directories
            if (mod_name != "shared" && mod_name != "." && mod_name != "..")
            {
                mods.push_back(mod_name);
            }
        }
    }

    return mods;
}

}  // anonymous namespace

// ============================================================================
// C++ functions called by Rust (defined in the CXX bridge)
// ============================================================================

namespace ctd {

// Get the load order - for UE4SS this reports installed mods
rust::Vec<PluginInfo> get_load_order()
{
    rust::Vec<PluginInfo> plugins;

    auto mods_dir = get_mods_directory();
    if (mods_dir.empty())
    {
        return plugins;
    }

    // First try to parse mods.txt for enabled status
    auto mods_txt = mods_dir / "mods.txt";
    auto mods_from_txt = parse_mods_txt(mods_txt);

    if (!mods_from_txt.empty())
    {
        uint32_t index = 0;
        for (const auto& [name, enabled] : mods_from_txt)
        {
            if (enabled)
            {
                PluginInfo info;
                info.name = rust::String(name);
                info.index = index++;
                info.is_light = false;  // UE4SS mods don't have light/full distinction
                plugins.push_back(info);
            }
        }
    }
    else
    {
        // Fall back to scanning directory
        auto mod_names = scan_mods_directory(mods_dir);
        uint32_t index = 0;
        for (const auto& name : mod_names)
        {
            PluginInfo info;
            info.name = rust::String(name);
            info.index = index++;
            info.is_light = false;
            plugins.push_back(info);
        }
    }

    return plugins;
}

// Get the game version string from the executable
rust::String get_game_version()
{
    auto exe_path = get_game_exe_path();
    auto version = get_file_version(exe_path);
    return rust::String(version);
}

}  // namespace ctd

// ============================================================================
// UE4SS Mod Implementation
// ============================================================================

class CTDCrashReporter : public RC::CppUserModBase
{
public:
    CTDCrashReporter() : CppUserModBase()
    {
        ModName = STR("CTDCrashReporter");
        ModVersion = STR("0.1.0");
        ModDescription = STR("Crash reporter for Oblivion Remastered - sends crash data to ctd.ezmode.games");
        ModAuthors = STR("ezmode.games");
    }

    ~CTDCrashReporter() override
    {
        // Shutdown the Rust crash handler
        ctd::shutdown();
    }

    auto on_unreal_init() -> void override
    {
        Output::send<LogLevel::Verbose>(STR("[CTD] Initializing crash reporter for Oblivion Remastered\n"));

        // Get actual UE version from UE4SS
        std::string ue_version = std::to_string(Version::Major) + "." + std::to_string(Version::Minor);

        // Get game version from executable
        std::string game_version = std::string(ctd::get_game_version());

        Output::send<LogLevel::Verbose>(STR("[CTD] Game version: {}, UE version: {}\n"),
                                        ensure_str(to_generic_string(game_version)),
                                        ensure_str(to_generic_string(ue_version)));

        // Initialize the Rust crash handler
        ctd::init("oblivion-remastered", game_version.c_str(), ue_version.c_str());

        Output::send<LogLevel::Verbose>(STR("[CTD] Crash reporter initialized\n"));
    }

    auto on_update() -> void override
    {
        // Called each frame - not needed for crash reporting
    }
};

#define CTD_MOD_API __declspec(dllexport)

extern "C"
{
    CTD_MOD_API RC::CppUserModBase* start_mod()
    {
        return new CTDCrashReporter();
    }

    CTD_MOD_API void uninstall_mod(RC::CppUserModBase* mod)
    {
        delete mod;
    }
}
