CTD Crash Reporter for Oblivion Remastered
==========================================
Version: 0.1.0

DESCRIPTION
-----------
Automatically captures crash reports when Oblivion Remastered crashes to desktop.
Reports are submitted to ctd.ezmode.games to help mod authors identify and fix
compatibility issues.

FEATURES
--------
- Captures crashes via Windows exception handling
- Collects your load order (esp/esm plugins) at crash time
- Submits anonymous crash reports with stack traces
- Helps identify crash patterns across the community

REQUIREMENTS
------------
- UE4SS 3.0.0 or newer (included with most mod setups)

INSTALLATION
------------
Vortex: Install normally, Vortex will handle placement.

MO2: Install normally, enable in the UE4SS mods tab.

Manual: Copy the CTDCrashReporter folder to:
  OblivionRemastered/Binaries/Win64/ue4ss/Mods/

Then add to mods.txt:
  CTDCrashReporter : 1

SOURCE CODE
-----------
https://github.com/ezmode-games/ctd

LICENSE
-------
AGPL-3.0-or-later
