# Aimon v1.0.0 Release Notes

Aimon is a terminal-based macOS system monitor written in Rust. This release reflects the current project state: an Apple Silicon oriented TUI monitor with live system metrics, multiple layouts, headless export modes, and release packaging for macOS.

## Highlights

- Interactive Ratatui terminal UI with startup, full, advanced, minimal, compact, battery, GPU, network, system health, and thermal layouts.
- macOS and Apple Silicon focused collection for CPU, memory, battery, thermal, process, network, disk I/O, power, energy, and carbon estimates.
- Session report shown before exit, including runtime, estimated Wh, CO₂ estimate, average/peak power, anomaly count, and top estimated energy offender.
- Headless output modes for one-shot JSON/CSV and streaming JSON, CSV, or Prometheus output.
- Theme switching, TOML configuration, bilingual documentation, and screenshot gallery under `images/`.
- Release packaging now uses the actual binary/package name: `aimon`.

## Download Assets

The release workflow publishes these macOS artifacts:

```text
dist/
├── aimon-v1.0.0-macos.tar.gz
└── aimon-v1.0.0-checksums.txt
```

The archive contains:

```text
aimon
README.txt
install.sh
uninstall.sh
```

## Install

```bash
# Download `aimon-v1.0.0-macos.tar.gz` from this release's Assets first.
tar -xzf aimon-v1.0.0-macos.tar.gz
./install.sh
```

Run with full `powermetrics` access:

```bash
sudo aimon
```

Run without root if you only need normal-user metrics:

```bash
aimon
```

## Verify

```bash
# Download `aimon-v1.0.0-checksums.txt` from this release's Assets first.
shasum -a 256 -c aimon-v1.0.0-checksums.txt
```

## Requirements

- macOS; the collectors rely on macOS system commands and APIs.
- Apple Silicon is the primary target for power and thermal metrics.
- `sudo` is recommended for complete `powermetrics` data.
- A modern terminal with Unicode support is recommended for best TUI rendering.

## CLI Quick Reference

```bash
aimon [options]

Options:
  -r, --refresh <SECONDS>     Refresh interval in seconds
      --refresh-ms <MILLIS>   Refresh interval in milliseconds; overrides --refresh
  -m, --minimal               Start in minimal display mode
  -c, --config <FILE>         Load a custom TOML config file
  -t, --theme <THEME>         Set initial theme
      --lang <LANG>           Set language value used by supported text paths: en, zh
      --json                  Output one JSON snapshot and exit
      --csv                   Output one CSV snapshot and exit
      --stream <FORMAT>       Stream output: json, csv, prometheus
  -h, --help                  Show help
  -V, --version               Show version
```

## Known Limits

- Complete power data depends on `powermetrics` permissions and availability.
- Some sensors are unavailable on certain Mac models or macOS versions.
- Per-process Wh is an estimate based on process CPU share and sampled package power, not a hardware-metered per-process reading.
- Battery runtime prediction is based on recent power draw and battery percentage, so it should be treated as an estimate.
- Small terminals may truncate some panels.

## Release Automation

- `.github/workflows/release.yml` builds on macOS when a `v*` tag is pushed or when manually dispatched.
- The workflow runs tests, builds the release binary, tests the binary, packages the archive, and publishes GitHub Release assets.
- GitHub Release notes are read directly from this `RELEASE.md` file.
- Local packaging can be run with `./scripts/build-release.sh` or `make package`.

## Inspiration And Licensing

Aimon is inspired by the excellent `mactop` project: https://github.com/metaspartan/mactop

`mactop` is distributed under the MIT License. Aimon is an independent Rust project and uses its own codebase and licensing (`MIT OR Apache-2.0` as declared in `Cargo.toml`). This attribution is included to clearly acknowledge the inspiration and upstream project.
