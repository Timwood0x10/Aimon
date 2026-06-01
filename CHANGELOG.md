# CHANGELOG

All notable changes to the system-alert project are documented here.

## [Unreleased] - 2026-05-31

### Changed
- Restructured API/module layout and UI layout/components.
- Refactored the main loop into an async, dual-threaded architecture; data collection now runs separately from UI rendering.
- Updated theme implementation.

### Fixed
- UI layouts for full, advanced, battery, GPU, compact, minimal, and thermals views.

### Added
- Collectors for DRAM, GPU, ANE, Thunderbolt, disk I/O, and terminal metrics.
- Headless output paths: `--json`, `--csv`, `--stream json|csv|prometheus`.
- Carbon tracking and an efficiency advisor panel.
- Session report overlay.
- Achievements, retro effects, time-travel snapshot views, and sonification components.
- I18n support (`--lang en|zh`) and notification manager.

## [0.2.0] - 2025-07-07

### Added
- Release automation: `scripts/`, `Makefile`, GitHub Actions workflows (CI + Release).
- README and README.zh-CN documentation.
- Project images for multiple layouts.

### Changed
- Repo branding/host adjusted to match current release docs.

## [0.1.0] - 2024-09-30

### Added
- Initial macOS system monitoring with Apple Silicon power metrics.
- Terminal UI layouts.
- Collector and CLI scaffolding.
