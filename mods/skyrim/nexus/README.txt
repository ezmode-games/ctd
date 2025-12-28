CTD - Crash to Desktop (Reporter)
==================================

We've all been there. CTD captures your crash info automatically and submits
it to a crash reporting service. Helps identify which mod is actually breaking
your game.

WHAT IT CAPTURES
----------------
- Exception code and address
- Stack trace with module offsets
- Faulting module name
- Complete mod load order
- Game version
- SKSE version

INSTALLATION
------------
1. Install SKSE64 (required)
2. Copy the SKSE folder to your Skyrim Special Edition/Data folder
3. Edit ctd.toml to configure the crash report server URL

CONFIGURATION
-------------
Edit Data/SKSE/Plugins/ctd.toml to set:
- url: The crash report server URL
- api_key: Optional authentication key
- timeout_secs: Request timeout

REQUIREMENTS
------------
- Skyrim Special Edition or Anniversary Edition
- SKSE64 (Script Extender)

SELF-HOSTING
------------
CTD is open source. You can run your own crash report server.
See: https://github.com/ezmode-games/ctd

LICENSE
-------
MIT License - See LICENSE file for details.

CREDITS
-------
ezmode.games
