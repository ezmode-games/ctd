# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-01-14

### Added
- Automated version bump workflow (`scripts/bump-version.ps1`)
- PDB symbol resolution for enhanced stack traces (#57)
- Fallout 3 and New Vegas support promoted to beta

### Fixed
- Clippy warnings for nested if statements (#58)

### Changed
- Rewrote README for clarity
- Updated architecture documentation

## [0.1.2] - 2026-01-14

### Added
- OpenAPI documentation for the API (#55)
- Config download endpoint (#54)
- API key management (#53)
- File hashing and DLL version extraction (#39)
- Status.json generation for external consumption (#49)
- Fallout 3 and New Vegas mod scaffolding (#44)

### Fixed
- UE4SS fingerprinting wired to crash handler (#52)
- Migrated Fallout mods to load_order_v2 API (#51)

### Changed
- Switched all packaging from .zip to .7z format (#56)

## [0.1.1] - 2025-12-28

### Added
- Skyrim SE (SKSE64) crash reporter
- Fallout 4 (F4SE) crash reporter
- Cyberpunk 2077 (RED4ext) crash reporter
- UE5/UE4SS crash reporter scaffold
- Per-mod release workflow with automatic CI builds (#31)
- Local build scripts for CMake mods

### Changed
- Restructured project as Cargo workspace (#26)
- Aligned Rust schema with API schema (#18)

[0.1.3]: https://github.com/ezmode-games/ctd/compare/skyrim-v0.1.2...skyrim-v0.1.3
[0.1.2]: https://github.com/ezmode-games/ctd/compare/v0.1.1...skyrim-v0.1.2
[0.1.1]: https://github.com/ezmode-games/ctd/releases/tag/v0.1.1
