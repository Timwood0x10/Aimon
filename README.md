# Aimon

[中文说明](README.zh-CN.md)

AIMON is a terminal-based macOS system monitor written in Rust. It focuses on Apple Silicon machines, showing live CPU, memory, battery, thermal, process, network, disk, power, energy, and carbon-related estimates in a TUI.

The application can run without root, but Apple `powermetrics` data is limited unless it is started with `sudo`.

## Current Status

This is an active local/system monitoring tool, not a polished packaged app. The repository currently supports:

- Interactive terminal UI with multiple layouts.
- macOS and Apple Silicon oriented power/thermal collection.
- Session energy and carbon estimates based on sampled package power.
- JSON, CSV, and streaming output modes.
- Theme switching and TOML configuration.

Some values are estimates. Per-process energy is calculated from process CPU share and package power; it is useful for comparison during one run, but it is not a hardware-metered per-process reading.

## Screenshots

| Layout | Preview |
| --- | --- |
| Startup page (`0`) | ![Startup](images/0.png) |
| Full layout (`1`) | ![Full](images/1.png) |
| Advanced layout (`2`) | ![Advanced](images/2.png) |
| Minimal layout (`3`) | ![Minimal](images/3.png) |
| Compact layout (`4`) | ![Compact](images/4.png) |
| Battery layout (`5`) | ![Battery](images/5.png) |
| GPU layout (`6`) | ![GPU](images/6.png) |
| Network layout (`7`) | ![Network](images/7.png) |
| System health layout (`8`) | ![System health](images/8.png) |
| Thermal layout (`9`) | ![Thermal](images/9.png) |
| Session report / exit screen | ![Session report](images/exit.png) |

## Requirements

- macOS. The project is designed around macOS system commands and Apple Silicon metrics.
- Rust toolchain for building from source.
- `sudo` is recommended for full `powermetrics` access.

## Build and Run

```bash
cargo build --release
sudo ./target/release/aimon
```

Development run:

```bash
sudo cargo run
```

Run without root if you only need the metrics available to a normal user:

```bash
cargo run
```

## CLI Options

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

Examples:

```bash
sudo cargo run -- --refresh 2
sudo cargo run -- --refresh-ms 500
sudo cargo run -- --theme mactop_green
cargo run -- --json
cargo run -- --stream prometheus
```

## Keyboard Controls

| Key | Action |
| --- | --- |
| `0` | Startup page |
| `1` | Full layout |
| `2` | Advanced layout |
| `3` | Minimal layout |
| `4` | Compact layout |
| `5` | Battery layout |
| `6` | GPU layout |
| `7` | Network layout |
| `8` | System health layout |
| `9` | Thermal layout |
| `d` | Storage layout |
| `h` / `←` | Previous layout |
| `l` / `→` / `Tab` | Next layout |
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `s` | Cycle process sort forward |
| `S` | Cycle process sort backward |
| `r` | Force redraw/refresh |
| `R` | Toggle session report |
| `t` | Cycle theme |
| `n` | Toggle notifications |
| `?` | Toggle help overlay |
| `q` | Show session report first; press again from the report to quit |
| `Ctrl+C` | Same quit event path as `q` |

## UI Features

- **Startup page**: centered dashboard-style entry page with hardware summary, self-checks, and masked serial number display.
- **Event badges**: top status badges such as idle power, thermal state, carbon level, and battery state.
- **Core matrix**: compact E-core/P-core activity cells in the full layout.
- **Power stack**: visual CPU/GPU/ANE/DRAM package power composition where available.
- **Process table**: process CPU, memory, and estimated accumulated Wh for the current run.
- **Efficiency advisor**: highlights high idle power and shows recent anomaly samples.
- **Session report**: runtime, total Wh, CO₂ estimate, average/peak power, anomaly count, and top estimated energy offender.
- **Battery runtime estimate**: when discharging, estimates remaining runtime from recent power draw.
- **Storage view**: mounted-volume bars, filesystem/removable labels, disk I/O, capacity trends, and a one-shot top-directory scan. If `dust` is installed it is used first; otherwise Aimon falls back to a bounded built-in scanner.

## Data Sources

Aimon uses a layered collector stack. Stable public tools remain the fallback path, while native macOS backends are treated as experimental until they are validated across more Apple Silicon machines:

- `sysinfo`: CPU usage, processes, memory, network interfaces, system metadata, and component temperatures where exposed.
- `powermetrics`: CPU/GPU/ANE/package power and CPU residency/frequency parsing; full data usually requires `sudo`.
- Mach `host_processor_info`: native per-core CPU usage on macOS when available.
- IOReport: experimental no-sudo CPU/GPU/ANE/DRAM power sampling when matching channels are available.
- Apple SMC: experimental temperature/fan enumeration; SMCWrite fan mode and target RPM are gated behind the `fan-control` Cargo feature, config enablement, and `--allow-fan-control`.
- IOKit IORegistry: experimental Apple GPU core count, max frequency, and frequency-table lookup when the platform exposes those properties.
- NSProcessInfo.thermalState: native thermal-state lookup through Foundation on macOS.
- `sysctl`: thermal pressure and model/CPU metadata used by fallback detection.
- `system_profiler`, `ioreg`, and `pmset`: GPU model details and battery/charger/capacity information.
- `iostat`: disk I/O sampling.
- `sysinfo::Disks`: mounted-volume disk capacity and usage percentages.
- Optional `dust`: one-shot top-directory size scan for the Storage view. If unavailable or timed out, Aimon uses its bounded built-in scanner.

IOHIDEventSystemClient temperature fallback is experimental. Fan writes are disabled in default builds; enabling them requires compiling with `--features fan-control`, setting `fan_control.enabled = true`, and passing `--allow-fan-control` at runtime.

For the concrete collector-by-collector implementation plan, see `docs/data-collection-roadmap.md`.

## Headless Output

One-shot JSON:

```bash
cargo run -- --json
```

One-shot CSV:

```bash
cargo run -- --csv
```

Streaming:

```bash
cargo run -- --stream json
cargo run -- --stream csv
cargo run -- --stream prometheus
```

## Configuration

The default config file is `config.toml`. A custom file can be passed with `--config`.

Important sections:

- `refresh_rate` and `minimal_mode` for basic runtime behavior.
- `[thresholds]` for CPU, memory, and temperature warning levels.
- `[display]` for optional panels, history size, and theme.
- `[notifications]` for notification enablement and cooldown.

Example:

```toml
refresh_rate = 1
minimal_mode = false

[display]
history_size = 60
theme = "mactop_green"

[notifications]
enabled = true
cooldown_seconds = 30
```

## Themes

The runtime `t` key cycles through the mactop-style foreground themes currently returned by the application:

- `mactop_green`
- `mactop_red`
- `mactop_blue`
- `mactop_yellow`
- `mactop_magenta`
- `mactop_cyan`
- `mactop_white`

The theme module also contains additional named theme constructors, but not every constructor is currently included in the runtime cycle.

## Privacy Notes

- The startup page masks the hardware serial number. It keeps only the first two and last two characters, with a short hash in the middle.
- Headless JSON/API output may still include whatever fields are exposed by `SystemData`; review output before sharing logs publicly.
- The tool reads local system metrics and does not require network access for normal monitoring.

## Inspiration

Aimon is inspired by the excellent [`mactop`](https://github.com/metaspartan/mactop) project. `mactop` is MIT licensed; Aimon is an independent Rust project and keeps this attribution to clearly acknowledge the inspiration.

## Limitations

- Full power metrics depend on macOS `powermetrics` permissions and availability.
- Some sensors are unavailable on some Macs or macOS versions.
- Per-process Wh is an estimate for the current session, not a direct hardware counter.
- Battery runtime prediction uses recent package/system power and battery percentage; it should be treated as an estimate.
- The UI is terminal-size dependent. Very small terminals may truncate panels.

## Troubleshooting

If power values are missing or zero, run with `sudo`:

```bash
sudo cargo run
```

Check `powermetrics` availability:

```bash
which powermetrics
sudo powermetrics --samplers cpu_power,gpu_power -n 1
```

If the UI looks cramped, enlarge the terminal or use a focused layout such as battery, GPU, network, health, or thermals.

If build checks fail because a wrapper such as `sccache` is restricted, run:

```bash
RUSTC_WRAPPER= cargo check
```

## Project Layout

```text
src/
├── main.rs              # TUI runtime and main event loop
├── cli.rs               # CLI parsing and keyboard input
├── config.rs            # TOML configuration
├── collectors/          # CPU, GPU, memory, process, battery, network, thermal collectors
├── carbon/              # Energy/carbon tracking and advisor rendering
├── api/                 # JSON, CSV, and Prometheus headless output
├── ui/                  # Ratatui layouts and components
├── history.rs           # Time-series history buffers
└── types.rs             # Shared data structures
```

## License

Apache 2.0
