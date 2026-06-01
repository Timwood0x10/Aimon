# Aimon Data Collection Implementation Status

This document records the real data paths that exist in the codebase today. It must not advertise a backend as stable unless the code actively uses it and the fallback behavior is clear.

## Ground Rules

- A metric must be labeled as measured, derived, estimated, experimental, or unavailable.
- A native Apple backend must fail closed and fall back without crashing.
- Fan control writes are implemented only behind the `fan-control` Cargo feature and require runtime `--allow-fan-control` consent plus config enablement.
- If a file only contains a stub, this document must say so.
- After data-source changes, run `make fmt`, `make check`, and targeted tests for the changed module.

## Runtime Pipeline

`SystemData` is defined in `src/types.rs` and is produced by `DataCollector::collect_all_data()` in `src/collectors/mod.rs`.

```text
src/main.rs
  └── DataCollector::collect_all_data()
        ├── refresh sysinfo state
        ├── collect system, memory, network, temperature, process, terminal data
        ├── collect CPU usage and power via Mach/IOReport/powermetrics/fallback
        ├── derive ANE and DRAM views from CPU/power metrics
        ├── collect GPU, Thunderbolt, disk I/O, thermal, performance, health data
        ├── collect battery data
        └── record carbon/session sample from package power
```

The same `SystemData` object is used by:

- TUI layouts under `src/ui/`.
- JSON/CSV/Prometheus output under `src/api/`.
- Session energy and carbon tracking under `src/carbon/`.

## Current Data Sources

| Area | Structure | Current path | Source | Status |
| --- | --- | --- | --- | --- |
| CPU usage | `CpuInfo` | `src/collectors/mod.rs` | Mach `host_processor_info`; fallback `sysinfo` | Native on macOS, fallback elsewhere |
| CPU/GPU/ANE/DRAM power | `CPUMetrics` | `src/collectors/mod.rs`, `src/collectors/backends/ioreport.rs` | IOReport; fallback `powermetrics`; final fallback estimate | Experimental native backend |
| CPU residency/frequency | `CPUMetrics` | `src/collectors/backends/powermetrics.rs` | `powermetrics --samplers cpu_power,gpu_power` | Measured when available |
| DRAM bandwidth | `DramInfo` | `src/collectors/mod.rs` | None | Not implemented; byte-rate fields remain `0.0` |
| GPU dynamic metrics | `GpuInfo` | `src/collectors/gpu.rs` | Cached `powermetrics` output | Partial parser coverage |
| GPU static metrics | `GpuInfo` | `src/collectors/gpu.rs`, `src/collectors/backends/iokit_gpu.rs` | IOKit IORegistry, `system_profiler`, `sysctl`, chip heuristics | Best effort with IOKit frequency table when exposed |
| Temperature | `TemperatureInfo` | `src/collectors/temperature.rs` | SMC; fallback IOHID; fallback `sysinfo::Components` | Experimental native backends with fallback |
| Thermal state | `ThermalInfo` | `src/collectors/thermal.rs` | Foundation `NSProcessInfo.thermalState` | Native on macOS when available |
| Thermal pressure | `ThermalInfo` | `src/collectors/thermal.rs` | `sysctl machdep.xcpm.cpu_thermal_level` | Best effort |
| Fan RPM/control | `ThermalInfo` | `src/collectors/thermal.rs`, `src/collectors/backends/smc.rs` | SMC read/write; fallback `powermetrics --samplers smc` for RPM | Reads by default; writes require feature + runtime consent |
| Battery | `BatteryInfo` | `src/battery_collector.rs` | `pmset`, `system_profiler`, `ioreg` | Best effort |
| Memory | `MemoryInfo` | `src/collectors/memory.rs` | `sysinfo` | Measured |
| Network | `NetworkInterface` | `src/collectors/network.rs` | `sysinfo` | Measured counters |
| Process table | `ProcessInfo` | `src/collectors/process.rs` | `sysinfo` | Measured |
| Disk I/O | `DiskIoInfo` | `src/collectors/disk_io.rs` | `iostat` | Command output parsing |
| Disk usage | `DiskUsageInfo` | `src/collectors/disk_usage.rs` | `sysinfo::Disks` | Measured mounted-volume capacity |
| Directory usage | `DirectoryUsageInfo` | `src/collectors/directory_usage.rs` | One-shot background `dust` scan; fallback bounded filesystem scan under common home folders | Partial by design when fallback depth/entry limits are hit |
| Thunderbolt | `ThunderboltInfo` | `src/collectors/thunderbolt.rs` | `system_profiler` | Command output parsing |
| System health | `SystemHealthInfo` | `src/collectors/health.rs` | `sysctl` and system commands | Partial |
| Carbon/session energy | `CarbonTracker` | `src/collectors/mod.rs`, `src/carbon/` | Integrated package watts over time | Estimate based on sampled power |

## Native Backend Status

### Active Or Fallback Backends

- `src/collectors/backends/mach.rs`: active macOS CPU usage backend. It computes per-core CPU usage from Mach tick deltas and falls back to `sysinfo` when sampling fails.
- `src/collectors/backends/ioreport.rs`: experimental macOS power backend. It dynamically loads IOReport symbols and falls back to `powermetrics` or estimates.
- `src/collectors/backends/smc.rs`: experimental SMC backend. It reads temperature and fan keys; SMCWrite fan mode/target RPM methods compile only with `fan-control`.
- `src/collectors/backends/iokit_gpu.rs`: experimental IORegistry backend for Apple GPU core count, max frequency, and frequency table properties.
- `src/collectors/backends/foundation.rs`: active macOS thermal-state backend through `objc2-foundation`.
- `src/collectors/backends/iohid.rs`: fallback temperature backend. It is used only when SMC returns no temperatures.
- `src/collectors/backends/powermetrics.rs`: command backend for CPU residency, frequency, and power fallback.
- `src/collectors/backends/sysinfo_backend.rs`: wrapper for basic `sysinfo` state used by tests and fallback logic.

### Not Implemented

- DRAM bandwidth estimation is not implemented. `DramInfo` bandwidth fields stay `0.0`.
- Per-field metadata is incomplete. `CpuInfo`, `GpuInfo`, `DramInfo`, and `ThermalInfo` carry metadata; `TemperatureInfo` and `BatteryInfo` do not yet carry field-level source metadata.

## Metadata Contract

`MetricSource`, `MetricConfidence`, and `MetricMeta` live in `src/types.rs`.

Current metadata rules:

- `CpuInfo.usage_meta` is `Mach` when Mach sampling succeeds and `Sysinfo` when fallback is used.
- `CpuInfo.power_meta` is `IoReport`, `Powermetrics`, or `Estimate` depending on the selected power path.
- `GpuInfo.meta` exists, but GPU parser coverage is still partial.
- `DramInfo.meta` is `Powermetrics` when a DRAM power line is parsed; otherwise it is `Estimate`.
- `ThermalInfo.state_meta` is `Foundation` when Foundation returns a state; otherwise unavailable.
- `ThermalInfo.fan_meta` is `Smc` or `Powermetrics` when fan RPM is found; otherwise unavailable.

Required next code work:

1. Add metadata to `TemperatureInfo` so SMC, IOHID, and sysinfo readings are distinguishable without encoding the source in the label.
2. Add metadata to `BatteryInfo` fields that mix `pmset`, `system_profiler`, and `ioreg`.
3. Add per-field power metadata if IOReport returns partial CPU/GPU/ANE/DRAM data.
4. Keep CSV output stable until column names for metadata are decided.

## Safety Contract

Fan-control safety rules protect all write paths:

- `fan-control` Cargo feature gates SMCWrite fan mode and target RPM methods.
- `--allow-fan-control` sets `FanControlConfig.runtime_allowed` only.
- `FanControlConfig.enabled` can come from config, and writes require both config enablement and runtime consent.
- `fan_control.target_rpm` is optional and must validate inside the configured safe range before writes.
- Default builds cannot write fan settings.

## Required Tests Before Claiming Stability

Current test status must remain green:

```bash
make fmt
RUSTC_WRAPPER= make check
RUSTC_WRAPPER= cargo test
```

Additional tests required before promoting native backends from experimental to stable:

- IOReport fixtures or mocked channel samples for CPU/GPU/ANE/DRAM partial-data cases.
- SMC tests that cover invalid key size, missing keys, and fanless systems without requiring hardware in CI.
- IOHID tests that prove invalid/null event data is ignored safely.
- Metadata tests that verify fallback data is not marked as measured.
- Manual tests on at least one fanless Apple Silicon Mac and one fan-equipped Apple Silicon Mac.

## Development Tasks

### Task A: Make Source Metadata Complete

Files:

- `src/types.rs`
- `src/collectors/temperature.rs`
- `src/battery_collector.rs`
- `src/api/formatters.rs`
- `src/ui/components.rs`

Steps:

1. Add metadata to `TemperatureInfo`.
2. Add metadata to `BatteryInfo` or split battery sub-readings into source-aware fields.
3. Update JSON output to include metadata.
4. Keep CSV unchanged unless a migration note is added.
5. Add tests for SMC, IOHID, sysinfo, `pmset`, `system_profiler`, and `ioreg` fallback labels.

Acceptance criteria:

- Every estimated or fallback temperature/battery value has source metadata.
- No UI label is the only place where the source is stored.

### Task B: Harden IOReport

Files:

- `src/collectors/backends/ioreport.rs`
- `src/collectors/mod.rs`
- `src/types.rs`

Steps:

1. Reject null `dlsym` results before transmuting symbols.
2. Record channel names and units at debug level when matching fails.
3. Represent partial power samples explicitly instead of relying only on `package_w > 0.0`.
4. Add tests for zero, partial, and unknown-unit channel samples.

Acceptance criteria:

- IOReport can fail at any symbol or channel lookup without undefined behavior or panic.
- Partial samples are not marked as fully measured.

### Task C: IOKit GPU Frequency Table

Files:

- `src/collectors/backends/mod.rs`
- `src/collectors/gpu.rs`

Status:

- `src/collectors/backends/iokit_gpu.rs` performs real IORegistry lookups.
- `src/collectors/gpu.rs` wires IOKit core count, max frequency, and frequency table into `GpuInfo`.
- UI GPU panels render max frequency and the exposed frequency table.

### Task D: SMCWrite Fan Control

Files:

- `src/config.rs`
- `src/cli.rs`
- `src/collectors/backends/smc.rs`

Status:

- Default builds return a clear `fan-control feature is not enabled` error for write requests.
- Feature builds write `F*Md` and `F*Tg` only after config enablement and runtime consent.
- Target RPM is validated against safe config bounds and SMC fan min/max data before writing.

## Documentation Rules

Every data-source change must update:

- `README.md` data source section.
- `README.zh-CN.md` data source section.
- `RELEASE.md` if release behavior changes.
- This document.

Do not use phrases such as “fully supports”, “complete”, “guaranteed”, or “mactop-level” unless the backend has tests, fallbacks, and documentation that prove the claim.
