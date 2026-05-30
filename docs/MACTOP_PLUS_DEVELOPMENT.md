# Mactop++ Rust 开发设计文档

本文档定义 `system-alert` 向“Rust 版 mactop++”演进的产品目标、目录结构、数据结构、分页体系、主题配色、开发步骤和验收标准。目标不是做一个普通系统监控器，而是做一个面向 macOS / Apple Silicon 的高密度、强视觉、专业级终端仪表盘。

## 1. 产品目标

### 1.1 核心定位

- 做一个 Rust 实现的 Apple Silicon cockpit，默认体验接近 `mactop`，信息密度和视觉效果超过 `mactop`。
- 保留当前项目已有的 API、JSON、CSV、Prometheus、通知、历史数据能力，但默认 TUI 改成更强的 mactop 风格。
- 支持无 sudo 基础模式和 sudo 高级模式。无 sudo 时展示 sysinfo / ioreg 可拿到的数据；sudo 时启用 `powermetrics` 采集 CPU/GPU/ANE/DRAM/package 详细指标。

### 1.2 必须保留的信息

- 系统：主机名、macOS 版本、kernel、uptime、芯片型号、CPU/GPU 核心数量。
- CPU：总使用率、每核心使用率、P-core / E-core 分组、频率、cluster 活跃度、历史曲线。
- GPU：使用率、频率、功耗、历史曲线。
- ANE：使用率或活跃度、功耗。
- 内存：总量、已用、可用、swap、memory pressure、压缩内存，如可采集。
- 功耗：CPU/GPU/ANE/DRAM/package power、趋势、峰值、平均值。
- 温度：CPU/GPU/SoC/电池温度、thermal state、thermal pressure。
- 风扇：当前 RPM、目标 RPM、最小/最大 RPM、模式。
- 电池：电量、充电状态、健康度、循环次数、电压、电流、适配器功率、剩余时间。
- 进程：PID、名称、CPU%、MEM、读写 IO、线程数，支持排序和滚动。
- 网络：接口、上下行速率、累计收发、包数、错误数，如可采集。
- 磁盘：读写速率、读写 ops、累计 IO。
- Thunderbolt：总线、设备、吞吐，如可采集。
- Terminal / PTY：PTY 使用量、`kern.tty.ptmx_max` 上限、使用率、TTY sysctl 快照、持有 PTY 最多的进程。

### 1.3 体验目标

- 默认首页一屏惊艳，像芯片仪表盘而不是表格堆叠。
- 数字不丢失，但展示分层：总览页看状态，分页页看细节。
- 终端尺寸变化时自动适配：大屏完整、小屏紧凑、极小屏 minimal。
- 键盘操作和分页必须清晰，底部状态栏持续提示。

## 2. 目标文件目录

当前项目已有 `src/collectors`、`src/ui/layouts`、`src/types.rs`、`src/history.rs`。后续应按下面结构整理，尽量增量重构，避免一次性大爆炸。

```text
src/
├── main.rs                         # 程序入口，只负责启动、事件循环、模式分发
├── lib.rs                          # 模块导出
├── cli.rs                          # CLI 参数、键盘输入事件
├── config.rs                       # 配置、主题名、默认页面、采集开关
├── types.rs                        # 对外统一数据模型 SystemData
├── history.rs                      # 环形历史数据、sparkline 数据
├── collectors/
│   ├── mod.rs                      # DataCollector 聚合器
│   ├── cpu.rs                      # sysinfo CPU 基础采集
│   ├── powermetrics.rs             # sudo 高级采集，解析 powermetrics
│   ├── gpu.rs                      # GPU 指标映射
│   ├── ane.rs                      # ANE 指标映射，新增
│   ├── dram.rs                     # DRAM 功耗、带宽指标
│   ├── memory.rs                   # 内存与 memory pressure
│   ├── process.rs                  # 进程列表、排序字段、IO 字段
│   ├── disk_io.rs                  # 磁盘 IO 速率
│   ├── network.rs                  # 网络接口与速率
│   ├── thermal.rs                  # thermal state、温度、风扇
│   ├── battery.rs                  # 替代或合并 battery_collector.rs
│   ├── thunderbolt.rs              # Thunderbolt 总线和设备
│   ├── terminal.rs                 # PTY/TTY/sysctl 轻量采集
│   └── system.rs                   # OS、芯片、uptime、硬件信息，新增
├── ui/
│   ├── mod.rs                      # UI 初始化、draw 分发
│   ├── theme.rs                    # 主题结构、主题注册表、颜色 token
│   ├── layout.rs                   # 通用区域切分工具
│   ├── components.rs               # 通用组件，逐步拆小
│   ├── widgets/
│   │   ├── mod.rs                  # 新增
│   │   ├── header.rs               # Hero Header
│   │   ├── metric_card.rs          # 指标卡片
│   │   ├── core_matrix.rs          # P/E core 矩阵
│   │   ├── power_stack.rs          # 功耗堆叠条
│   │   ├── process_table.rs        # 可滚动进程表
│   │   ├── sparkline.rs            # 统一 sparkline
│   │   ├── heatmap.rs              # 芯片热力图
│   │   └── status_bar.rs           # 底部快捷键和状态
│   └── layouts/
│       ├── mod.rs                  # LayoutType / PageType
│       ├── overview.rs             # 新默认首页，mactop++ cockpit
│       ├── cpu.rs                  # CPU 深挖页，新增
│       ├── gpu.rs                  # GPU / ANE 深挖页
│       ├── memory.rs               # Memory / DRAM 深挖页，新增
│       ├── process.rs              # Process 深挖页，新增
│       ├── io.rs                   # Disk / Network / Thunderbolt 页，新增
│       ├── battery.rs              # Battery / Power 页
│       ├── health.rs               # Thermal / Fan / System Health 页
│       ├── compact.rs              # 小屏紧凑页
│       └── minimal.rs              # 极小屏 fallback
└── api/
    ├── mod.rs
    ├── export.rs
    ├── formatters.rs
    ├── prometheus.rs
    └── server.rs
```

## 3. 数据结构设计

### 3.1 顶层数据模型

`SystemData` 是 UI、API、导出共享的唯一快照。采集器可以内部拆分，但最终必须汇聚成这个结构。

```rust
pub struct SystemData {
    pub timestamp: DateTime<Utc>,
    pub collection_mode: CollectionMode,
    pub system: SystemInfo,
    pub chip: ChipInfo,
    pub cpu: CpuInfo,
    pub gpu: GpuInfo,
    pub ane: AneInfo,
    pub memory: MemoryInfo,
    pub dram: DramInfo,
    pub power: PowerInfo,
    pub thermal: ThermalInfo,
    pub battery: BatteryInfo,
    pub processes: Vec<ProcessInfo>,
    pub network: NetworkInfo,
    pub disk: DiskIoInfo,
    pub thunderbolt: ThunderboltInfo,
    pub terminal: TerminalInfo,
    pub health: SystemHealth,
}

pub enum CollectionMode {
    Basic,      // 不需要 sudo
    Enhanced,   // 部分 macOS 原生命令可用
    SudoPower,  // powermetrics 可用
}
```

### 3.2 芯片与系统信息

```rust
pub struct SystemInfo {
    pub host_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,
    pub boot_time: Option<DateTime<Utc>>,
}

pub struct ChipInfo {
    pub name: String,              // Apple M1/M2/M3/M4 Pro/Max/Ultra
    pub architecture: String,      // arm64
    pub cpu_core_count: usize,
    pub p_core_count: usize,
    pub e_core_count: usize,
    pub gpu_core_count: usize,
    pub neural_engine_core_count: Option<usize>,
    pub memory_bandwidth_gbps: Option<f32>,
}
```

### 3.3 CPU 数据结构

```rust
pub struct CpuInfo {
    pub average_usage: f32,
    pub user_usage: Option<f32>,
    pub system_usage: Option<f32>,
    pub idle_usage: Option<f32>,
    pub p_cluster: ClusterInfo,
    pub e_cluster: ClusterInfo,
    pub cores: Vec<CpuCoreInfo>,
    pub load_average: Option<[f32; 3]>,
}

pub struct ClusterInfo {
    pub name: ClusterKind,
    pub usage: f32,
    pub active_residency: Option<f32>,
    pub idle_residency: Option<f32>,
    pub frequency_mhz: Option<f32>,
    pub power_watts: Option<f32>,
}

pub enum ClusterKind {
    Performance,
    Efficiency,
}

pub struct CpuCoreInfo {
    pub id: usize,
    pub kind: ClusterKind,
    pub usage: f32,
    pub frequency_mhz: Option<f32>,
    pub active_residency: Option<f32>,
    pub temperature_celsius: Option<f32>,
}
```

### 3.4 GPU / ANE / DRAM / Power

```rust
pub struct GpuInfo {
    pub usage: Option<f32>,
    pub frequency_mhz: Option<f32>,
    pub power_watts: Option<f32>,
    pub core_count: usize,
}

pub struct AneInfo {
    pub usage: Option<f32>,
    pub power_watts: Option<f32>,
    pub active_residency: Option<f32>,
}

pub struct DramInfo {
    pub power_watts: Option<f32>,
    pub bandwidth_read_gbps: Option<f32>,
    pub bandwidth_write_gbps: Option<f32>,
}

pub struct PowerInfo {
    pub package_watts: Option<f32>,
    pub cpu_watts: Option<f32>,
    pub gpu_watts: Option<f32>,
    pub ane_watts: Option<f32>,
    pub dram_watts: Option<f32>,
    pub average_package_watts: Option<f32>,
    pub peak_package_watts: Option<f32>,
}
```

### 3.5 Memory / Thermal / Battery

```rust
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub wired_bytes: Option<u64>,
    pub compressed_bytes: Option<u64>,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub usage_percent: f32,
    pub pressure: MemoryPressure,
}

pub enum MemoryPressure {
    Normal,
    Warning,
    Critical,
    Unknown,
}

pub struct ThermalInfo {
    pub state: ThermalState,
    pub pressure: Option<f32>,
    pub sensors: Vec<TemperatureSensor>,
    pub fans: Vec<FanInfo>,
}

pub struct TemperatureSensor {
    pub label: String,
    pub component: ThermalComponent,
    pub temperature_celsius: f32,
    pub critical_celsius: Option<f32>,
}

pub enum ThermalComponent {
    Cpu,
    Gpu,
    Soc,
    Battery,
    Ambient,
    Unknown,
}
```

### 3.6 Process / Network / Disk

```rust
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub command: Option<String>,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub disk_read_bytes_per_sec: Option<f64>,
    pub disk_write_bytes_per_sec: Option<f64>,
    pub thread_count: Option<u32>,
}

pub enum ProcessSortBy {
    Cpu,
    Memory,
    DiskRead,
    DiskWrite,
    Pid,
    Name,
}

pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterfaceInfo>,
    pub total_rx_bytes_per_sec: f64,
    pub total_tx_bytes_per_sec: f64,
}

pub struct NetworkInterfaceInfo {
    pub name: String,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub packets_rx_per_sec: Option<f64>,
    pub packets_tx_per_sec: Option<f64>,
}
```

### 3.7 Terminal / PTY / TTY 数据结构

macOS 上 PTY/TTY 能反映终端会话、远程登录、shell、开发工具、multiplexer、伪终端泄漏等状态。这个信息不如 CPU/GPU 抢眼，但对“了解 Mac 的一举一动”很有价值，应该作为轻量后台任务持续采集。

```rust
pub struct TerminalInfo {
    pub pty: PtyInfo,
    pub tty_sysctl: TtySysctlInfo,
    pub sessions: Vec<TerminalSessionInfo>,
    pub top_processes: Vec<TerminalProcess>,
    pub last_updated: Option<DateTime<Utc>>,
    pub collection_error: Option<String>,
}

pub struct PtyInfo {
    pub active_count: u32,
    pub local_count: u32,
    pub remote_count: u32,
    pub max_count: Option<u32>,              // sysctl kern.tty.ptmx_max
    pub usage_percent: Option<f32>,          // active_count / max_count * 100
    pub pressure: PtyPressure,
}

pub enum PtyPressure {
    Normal,      // < 50%
    Elevated,    // 50% - 75%
    High,        // 75% - 90%
    Critical,    // >= 90%
    Unknown,
}

pub struct TtySysctlInfo {
    pub ptmx_max: Option<u32>,               // kern.tty.ptmx_max
    pub raw: Vec<SysctlMetric>,              // 其他 kern.tty.* 可用项
}

pub struct SysctlMetric {
    pub key: String,
    pub value: String,
    pub source: SysctlSource,
}

pub enum SysctlSource {
    KernTty,
    Kern,
    User,
    Unknown,
}

pub struct TerminalSessionInfo {
    pub tty: String,                         // ttys000 / pts/0 / console
    pub user: Option<String>,
    pub pid: Option<u32>,
    pub command: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub is_local: bool,
    pub is_remote: bool,
}

pub struct TerminalProcess {
    pub pid: u32,
    pub name: String,
    pub pty_count: u32,
    pub user: Option<String>,
}
```

必须采集的 sysctl key：

| Key | 字段 | 说明 |
|---|---|---|
| `kern.tty.ptmx_max` | `pty.max_count` / `tty_sysctl.ptmx_max` | 系统允许的 PTY 最大数量，可用于计算 PTY pressure |

建议尝试采集的相关信息：

| 命令 | 用途 | 要求 |
|---|---|---|
| `sysctl kern.tty` | 枚举当前系统暴露的 TTY sysctl key | 可能因系统版本无输出，必须容忍 |
| `sysctl kern.tty.ptmx_max` | 读取 PTY 上限 | 可能不存在，必须返回 `None` |
| `who` | 当前登录/终端会话 | 轻量，无 sudo |
| `w -h` | 会话、用户、命令概览 | 轻量，无 sudo |
| `lsof -nP /dev/ttys* /dev/ptmx` | 进程持有 TTY/PTY 情况 | 较重，低频执行 |
| `/bin/ls /dev/ttys*` | 活跃 TTY 设备粗略计数 | fallback |

采集规则：

- `kern.tty.ptmx_max` 是优先级最高的轻量指标，先用 `sysctl -n kern.tty.ptmx_max` 获取。
- `sysctl kern.tty` 没有输出不是错误，记录为空即可。
- `lsof` 不允许每秒执行，默认 10 - 30 秒一次，或者仅在 Terminal 页打开时执行。
- PTY 使用率超过 75% 在 Overview 显示 warning，超过 90% 显示 critical。
- `TerminalInfo.collection_error` 只用于 debug/详情页，不要在首页刷屏。

### 3.8 历史数据

历史数据用于 sparkline、趋势、峰值、均值。必须使用环形缓冲，避免长时间运行内存增长。

```rust
pub struct HistoryData {
    pub max_points: usize,
    pub cpu_usage: RingSeries<f32>,
    pub p_cluster_usage: RingSeries<f32>,
    pub e_cluster_usage: RingSeries<f32>,
    pub gpu_usage: RingSeries<f32>,
    pub package_power: RingSeries<f32>,
    pub memory_usage: RingSeries<f32>,
    pub temperature: RingSeries<f32>,
    pub network_rx: RingSeries<f64>,
    pub network_tx: RingSeries<f64>,
    pub disk_read: RingSeries<f64>,
    pub disk_write: RingSeries<f64>,
    pub pty_usage: RingSeries<f32>,
}

pub struct RingSeries<T> {
    pub values: VecDeque<T>,
    pub capacity: usize,
}
```

## 4. 采集架构

### 4.1 采集分层

- `BasicCollector`：通过 `sysinfo` 和 libc 获取 CPU、内存、进程、网络基础信息，不需要 sudo。
- `MacOsCollector`：通过 `ioreg`、`pmset`、`sysctl`、`vm_stat` 获取 macOS 专属信息。
- `PowermetricsCollector`：通过 `sudo powermetrics -n 1 --samplers ...` 获取 Apple Silicon 高级指标。
- `TerminalCollector`：通过 `sysctl kern.tty.ptmx_max`、`who`、`w`、低频 `lsof` 获取 PTY/TTY 状态。
- `DataCollector`：统一调度所有 collector，合并为 `SystemData`。

### 4.2 powermetrics 策略

- 不在每个 UI frame 调用 `powermetrics`，按 `power_refresh_rate` 独立刷新，默认 2 秒。
- 解析结果放入缓存，UI 每秒刷新时使用最近一次成功数据。
- 超时必须降级，不允许卡住 TUI。建议 timeout 1500ms - 2500ms。
- 如果未 sudo 或命令失败，在 UI 显示 `Enhanced metrics locked: run with sudo`，但基础页面继续工作。

### 4.3 建议 samplers

```text
powermetrics -n 1 --samplers cpu_power,gpu_power,ane_power,thermal
```

不同 macOS 版本 sampler 名称可能差异，解析器必须容忍缺字段。

### 4.4 TerminalCollector async 轻量采集策略

PTY/TTY 指标需要密集展示，但采集必须轻量，不能让 TUI 卡顿。实现方式建议是独立 async task + 缓存快照。

```rust
pub struct TerminalCollector {
    cache: Arc<RwLock<TerminalInfo>>,
    sysctl_interval: Duration,   // 默认 5s
    session_interval: Duration,  // 默认 5s
    lsof_interval: Duration,     // 默认 30s
    command_timeout: Duration,   // 默认 800ms
}

impl TerminalCollector {
    pub async fn spawn(self, shutdown: CancellationToken) -> JoinHandle<()>;
    pub async fn snapshot(&self) -> TerminalInfo;
}
```

任务分工：

- 快路径：`sysctl -n kern.tty.ptmx_max`、`who`、`w -h`，默认 5 秒刷新。
- 慢路径：`lsof -nP /dev/ttys* /dev/ptmx` 或等价命令，默认 30 秒刷新。
- UI 路径：只读取 `Arc<RwLock<TerminalInfo>>` 缓存，不直接执行命令。
- 超时策略：所有外部命令用 `tokio::process::Command` + `timeout`，超时后保留上一份成功快照。
- 降级策略：sysctl key 不存在时 `max_count = None`，PTY pressure 显示 `Unknown`，但 active/session 仍展示。

推荐命令实现：

```rust
async fn read_ptmx_max() -> Option<u32> {
    run_command("sysctl", &["-n", "kern.tty.ptmx_max"], 800).await
        .ok()
        .and_then(|stdout| stdout.trim().parse::<u32>().ok())
}

async fn read_kern_tty_sysctls() -> Vec<SysctlMetric> {
    run_command("sysctl", &["kern.tty"], 800).await
        .map(parse_sysctl_lines)
        .unwrap_or_default()
}
```

注意事项：

- 当前 `src/collectors/terminal.rs` 使用阻塞 `std::process::Command`，后续要迁移到 `tokio::process::Command`。
- 现有 fallback `Command::new("lsof").args(["|", "grep", "ptmx"])` 不会执行 shell 管道，应该删除或改成 Rust 内部过滤。
- `lsof` 输出可能很大，只保留 top 10 或 top 20，避免 UI 和 JSON 过重。
- `kern.tty.ptmx_max` 在不同 macOS 版本可能不存在，不能作为必定存在的字段。

## 5. 分页与键盘操作

### 5.1 页面列表

分页是核心能力，不能只做一个 full layout。建议页面顺序如下：

| 快捷键 | 页面 | 文件 | 目的 |
|---|---|---|---|
| `1` | Overview | `src/ui/layouts/overview.rs` | 默认首页，mactop++ cockpit |
| `2` | CPU | `src/ui/layouts/cpu.rs` | P/E core、频率、residency、CPU 历史 |
| `3` | GPU/ANE | `src/ui/layouts/gpu.rs` | GPU、ANE、图形功耗、活跃度 |
| `4` | Memory | `src/ui/layouts/memory.rs` | RAM、swap、pressure、DRAM 功耗 |
| `5` | Processes | `src/ui/layouts/process.rs` | 可滚动进程表、排序、过滤 |
| `6` | IO | `src/ui/layouts/io.rs` | Disk、Network、Thunderbolt、Terminal/PTY |
| `7` | Battery | `src/ui/layouts/battery.rs` | 电池、适配器、续航、充电 |
| `8` | Health | `src/ui/layouts/health.rs` | 温度、风扇、thermal state |
| `9` | Compact | `src/ui/layouts/compact.rs` | 小屏高密度模式 |
| `0` | Minimal | `src/ui/layouts/minimal.rs` | 极简 fallback |

### 5.2 全局快捷键

| 快捷键 | 行为 |
|---|---|
| `q` / `Ctrl+C` | 退出 |
| `Tab` / `Shift+Tab` | 下一页 / 上一页 |
| `1..9`, `0` | 直接跳转页面 |
| `t` | 切换主题 |
| `r` | 立即刷新 |
| `s` | 切换进程排序字段 |
| `↑/↓` | 进程页滚动 |
| `PgUp/PgDown` | 进程页快速滚动 |
| `/` | 进入进程过滤输入 |
| `Esc` | 退出过滤或弹层 |
| `h` / `?` | 快捷键帮助弹窗 |
| `p` | 暂停/继续刷新 |
| `d` | 切换详细/紧凑密度 |

### 5.3 `LayoutType` 目标定义

```rust
pub enum LayoutType {
    Overview,
    Cpu,
    Gpu,
    Memory,
    Processes,
    Io,
    Battery,
    Health,
    Compact,
    Minimal,
}
```

旧的 `Full` 可以保留为 `Overview` 的别名，避免配置兼容问题。

## 6. 页面设计

### 6.1 Overview 默认首页

首页必须形成“芯片仪表盘”观感。

```text
┌ Apple M3 Pro ─ macOS 15.x ─ 14C/18G ─ Uptime 3d 02h ─ 68°C ─ 21.4W ┐
│ CPU ███████░░ 74%  GPU ███░░░░░ 31%  ANE ██░░░░░░ 12%  MEM █████░ 61% │
├ E-CORES ────────────────┬ P-CORES ────────────────────────────────────┤
│ E0 ▇▇▇░ 22%  972MHz     │ P0 ████████ 88% 3228MHz                     │
│ E1 ▇▇░░ 18%  972MHz     │ P1 ██████░░ 63% 3100MHz                     │
├ POWER STACK ────────────┼ MEMORY / DRAM ──────────────────────────────┤
│ CPU 8.2W GPU 4.1W       │ Used 18.4G / 36G  Pressure: Nominal          │
│ ANE 0.8W DRAM 1.8W      │ Swap 1.2G  DRAM 1.8W                        │
├ PROCESS TOP ────────────┼ IO / NETWORK ───────────────────────────────┤
│ PID   CPU  MEM  NAME    │ Disk R/W 120MB/s 32MB/s                     │
│ ...                     │ Net ↓18MB/s ↑2MB/s  PTY 42/127              │
└ 1 overview 2 cpu 3 gpu 4 mem 5 proc 6 io 7 batt 8 health t theme q quit ┘
```

布局规则：

- 高度 `< 24` 时隐藏进程和 IO 明细，只保留 Hero、core matrix、power/memory。
- 宽度 `< 100` 时 P/E core 矩阵改为单列。
- 宽度 `< 80` 时自动切到 `Compact`。
- 宽度 `< 60` 或高度 `< 16` 时自动切到 `Minimal`。

### 6.2 CPU 页

- 左侧：P-core / E-core 大矩阵，每个核心显示 usage、频率、温度。
- 右侧：cluster active/idle residency、load average、CPU power。
- 底部：CPU、P-cluster、E-cluster 三条历史 sparkline。
- 支持按 `d` 切换详细模式：详细模式展示每核心更多数字；紧凑模式展示热力块。

### 6.3 GPU/ANE 页

- 顶部：GPU usage、GPU freq、GPU power、ANE usage、ANE power。
- 中部：GPU / ANE 历史曲线。
- 底部：相关高 GPU/CPU 进程列表。
- 如果 ANE 数据不可用，显示 `ANE metrics unavailable on this macOS/powermetrics sampler`。

### 6.4 Memory 页

- 展示 RAM 总量、使用、可用、wired、compressed、swap。
- 展示 memory pressure 状态，用颜色表达：Normal 绿色、Warning 黄色、Critical 红色。
- 展示 DRAM power 和内存趋势。

### 6.5 Processes 页

- 表格列：PID、CPU%、MEM、READ/s、WRITE/s、THREADS、NAME。
- 支持 `s` 切换排序：CPU -> MEM -> READ -> WRITE -> PID -> NAME。
- 支持 `/` 过滤进程名称。
- 支持滚动，底部显示 `showing 1-30 / 412`。

### 6.6 IO 页

- 左侧 Disk：读写速率、ops、历史曲线。
- 右侧 Network：每接口上下行、总上下行、历史曲线。
- 中部 Terminal / PTY：`active_count / kern.tty.ptmx_max`、PTY pressure、local/remote session 数。
- 底部 Thunderbolt：bus、device、rx/tx，如无数据则隐藏或显示 unavailable。
- Terminal 明细只展示有效信息：top PTY processes、当前 `who` 会话、TTY sysctl 快照。
- Overview 只显示一行高密度摘要，例如 `PTY 42/127 33% · local 8 · remote 1 · top: zsh(12)`。

### 6.7 Battery 页

- 电量大进度条，充电/放电状态。
- 健康度、循环次数、当前容量、设计容量。
- 电压、电流、适配器功率、剩余时间。
- 显示功耗趋势，估算剩余续航。

### 6.8 Health 页

- Thermal state 大状态卡：Nominal / Fair / Serious / Critical。
- 温度传感器表：label、component、temp、critical。
- 风扇表：id、current、target、min、max、mode。
- 异常状态必须高亮，Critical 使用红色边框和闪烁感样式，但不要真的频繁闪屏。

## 7. 主题与配色规范

### 7.1 主题 token

现有 `Theme` 字段偏少。目标主题结构建议扩展为 token 化，不要在组件里硬编码颜色。

```rust
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub panel_bg: Color,
    pub panel_border: Color,
    pub panel_border_active: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub accent_alt: Color,
    pub cpu_p: Color,
    pub cpu_e: Color,
    pub gpu: Color,
    pub ane: Color,
    pub memory: Color,
    pub dram: Color,
    pub power: Color,
    pub thermal: Color,
    pub battery: Color,
    pub network_rx: Color,
    pub network_tx: Color,
    pub disk_read: Color,
    pub disk_write: Color,
    pub ok: Color,
    pub warning: Color,
    pub critical: Color,
}
```

### 7.2 默认主题：`mactop_pro`

默认主题要专业、清晰、接近 mactop 的绿色科技感，但比原版更精致。

| Token | RGB | 用途 |
|---|---:|---|
| `bg` | `#07110B` | 全局背景，接近黑绿 |
| `panel_bg` | `#0B1710` | 面板背景 |
| `panel_border` | `#1F4D32` | 普通边框 |
| `panel_border_active` | `#5CFF8A` | 当前重点面板边框 |
| `fg` | `#D8FFE1` | 主文字 |
| `muted` | `#6FA77C` | 次级文字 |
| `accent` | `#5CFF8A` | 主题强调绿 |
| `accent_alt` | `#7CFFCB` | 青绿色强调 |
| `cpu_p` | `#FFD166` | P-core 金色，表达性能 |
| `cpu_e` | `#66E3A2` | E-core 绿色，表达能效 |
| `gpu` | `#64D2FF` | GPU 蓝色 |
| `ane` | `#C084FC` | ANE 紫色 |
| `memory` | `#7CFFCB` | 内存青绿 |
| `dram` | `#2DD4BF` | DRAM 蓝绿 |
| `power` | `#FFB86B` | 功耗橙色 |
| `thermal` | `#FF6B6B` | 温度红色 |
| `battery` | `#A3FF12` | 电池亮绿 |
| `network_rx` | `#38BDF8` | 下载蓝色 |
| `network_tx` | `#FB7185` | 上传玫红 |
| `disk_read` | `#93C5FD` | 磁盘读 |
| `disk_write` | `#FBBF24` | 磁盘写 |
| `ok` | `#5CFF8A` | 正常状态 |
| `warning` | `#FFE066` | 警告状态 |
| `critical` | `#FF3864` | 严重状态 |

### 7.3 炫酷主题：`cyberpunk_plus`

适合截图和宣传，颜色更炸，但要保持可读性。

| Token | RGB |
|---|---:|
| `bg` | `#050510` |
| `panel_bg` | `#0B0B1F` |
| `panel_border` | `#243B80` |
| `panel_border_active` | `#00F5FF` |
| `fg` | `#EEF2FF` |
| `muted` | `#7B8AAD` |
| `accent` | `#00F5FF` |
| `accent_alt` | `#FF2BD6` |
| `cpu_p` | `#FFB000` |
| `cpu_e` | `#00FF9C` |
| `gpu` | `#00D9FF` |
| `ane` | `#BD00FF` |
| `memory` | `#00FFCC` |
| `dram` | `#2AFADF` |
| `power` | `#FF8A00` |
| `thermal` | `#FF2E63` |
| `battery` | `#B6FF00` |
| `network_rx` | `#00A3FF` |
| `network_tx` | `#FF4ECD` |
| `disk_read` | `#7DD3FC` |
| `disk_write` | `#FACC15` |
| `ok` | `#00FF9C` |
| `warning` | `#FFE600` |
| `critical` | `#FF1744` |

### 7.4 高级主题：`gold_pro`

适合专业、低调、长时间盯屏。

| Token | RGB |
|---|---:|
| `bg` | `#0E0D0A` |
| `panel_bg` | `#17140E` |
| `panel_border` | `#5A431B` |
| `panel_border_active` | `#FFD166` |
| `fg` | `#F3E8C8` |
| `muted` | `#A8905E` |
| `accent` | `#FFD166` |
| `accent_alt` | `#FFB86B` |
| `cpu_p` | `#FFD166` |
| `cpu_e` | `#B8F28B` |
| `gpu` | `#8ECAE6` |
| `ane` | `#CDB4DB` |
| `memory` | `#B8F2E6` |
| `dram` | `#94D2BD` |
| `power` | `#F4A261` |
| `thermal` | `#E76F51` |
| `battery` | `#C5F86F` |
| `network_rx` | `#90E0EF` |
| `network_tx` | `#FFAFCC` |
| `disk_read` | `#A2D2FF` |
| `disk_write` | `#FFC857` |
| `ok` | `#95D5B2` |
| `warning` | `#FFD166` |
| `critical` | `#EF476F` |

### 7.5 Matrix 主题：`matrix_neon`

适合复古极客风。

| Token | RGB |
|---|---:|
| `bg` | `#020804` |
| `panel_bg` | `#06120A` |
| `panel_border` | `#14532D` |
| `panel_border_active` | `#39FF14` |
| `fg` | `#C7FFD1` |
| `muted` | `#4ADE80` |
| `accent` | `#39FF14` |
| `accent_alt` | `#00FFB3` |
| `cpu_p` | `#D9FF00` |
| `cpu_e` | `#39FF14` |
| `gpu` | `#00FFB3` |
| `ane` | `#80FFDB` |
| `memory` | `#00F5A0` |
| `dram` | `#00D084` |
| `power` | `#E7FF5E` |
| `thermal` | `#FF5555` |
| `battery` | `#7CFF00` |
| `network_rx` | `#00E5FF` |
| `network_tx` | `#FFD60A` |
| `disk_read` | `#A7F3D0` |
| `disk_write` | `#FDE047` |
| `ok` | `#39FF14` |
| `warning` | `#FDE047` |
| `critical` | `#FF3131` |

### 7.6 颜色使用规则

- P-core 永远使用暖色，E-core 永远使用绿色或冷绿色，帮助用户形成认知。
- 功耗使用橙色，温度使用红色，电池使用亮绿，网络上下行必须不同色。
- 警告阈值：正常用 `ok`，超过 warning 用 `warning`，超过 critical 用 `critical`。
- 背景不要纯黑，使用轻微色相背景；边框要比背景亮但不能抢主数据。
- 不同主题只改 token，不允许组件自行选择固定 RGB。

## 8. 配置设计

`config.toml` 建议扩展：

```toml
refresh_rate = 1
power_refresh_rate = 2
default_layout = "overview"
theme = "mactop_pro"
language = "zh"

[collectors]
enable_powermetrics = true
enable_ioreg = true
enable_process_io = true
enable_thunderbolt = true
enable_terminal = true
powermetrics_timeout_ms = 2000
terminal_sysctl_refresh_seconds = 5
terminal_lsof_refresh_seconds = 30
terminal_command_timeout_ms = 800

[display]
density = "normal"           # compact, normal, detailed
show_borders = true
show_sparklines = true
show_unicode_blocks = true
auto_compact = true
history_size = 120

[process]
sort_by = "cpu"
limit = 30
show_command = false

[thresholds]
cpu_warning = 75.0
cpu_critical = 90.0
memory_warning = 75.0
memory_critical = 90.0
temperature_warning = 75.0
temperature_critical = 90.0
package_power_warning = 30.0
package_power_critical = 45.0
```

## 9. 开发步骤

### Phase 0：基线整理

1. 跑通当前 `cargo check`，记录现有 warning 和失败点。
2. 截图当前 UI，作为改造前基线。
3. 确认当前 `src/collectors` 中 `gpu.rs`、`dram.rs`、`disk_io.rs`、`thunderbolt.rs` 是否已经接入 `DataCollector`。
4. 把现有 `Full`、`GpuFocus`、`BatteryFocus` 页面标记为 legacy-compatible 页面，不急着删除。
5. 检查 `src/collectors/terminal.rs` 当前阻塞采集和错误 fallback，记录迁移点。

验收：不改变功能，项目仍能启动。

### Phase 1：页面系统升级

1. 在 `src/ui/layouts/mod.rs` 增加新的 `LayoutType`：`Overview`、`Cpu`、`Memory`、`Processes`、`Io`、`Health`。
2. 保留旧名称解析：`full -> overview`、`battery -> battery`、`network -> io`。
3. 修改键盘输入，让 `1..9` 和 `0` 可以跳转页面。
4. 底部状态栏显示所有页面快捷键。
5. 默认页面改成 `Overview`。

验收：可以分页，页面切换不崩，状态栏提示准确。

### Phase 2：主题系统升级

1. 扩展 `Theme` token，不删除旧字段，先兼容旧组件。
2. 新增主题：`mactop_pro`、`cyberpunk_plus`、`gold_pro`、`matrix_neon`。
3. 建立 `Theme::from_name(name)` 注册表。
4. 所有新组件只使用 token，不写死 RGB。
5. `t` 切换主题时按固定顺序循环。

验收：四套主题可切换，颜色符合本文档表格。

### Phase 3：Overview 首页

1. 新建 `src/ui/layouts/overview.rs`。
2. 新建或抽出 widgets：`header.rs`、`metric_card.rs`、`core_matrix.rs`、`power_stack.rs`、`process_table.rs`、`status_bar.rs`。
3. 首页布局按 Hero Header、Metric Row、Core Matrix、Power/Memory、Process/IO、Status Bar 六层实现。
4. 加入终端尺寸判断，自动 fallback 到 compact/minimal。
5. 优先展示已有数据，缺失字段显示 `--`，不要 panic。

验收：首页第一眼有 mactop cockpit 感，信息完整，缺高级数据时仍优雅。

### Phase 4：数据模型补齐

1. 扩展 `types.rs`，补齐 `ChipInfo`、`ClusterInfo`、`GpuInfo`、`AneInfo`、`DramInfo`、`PowerInfo`。
2. 写转换层，避免一次性改爆所有旧代码：旧字段继续能被老页面读取。
3. `HistoryData` 增加 GPU、power、disk、network、temperature 历史序列。
4. API JSON 输出包含新增字段。

验收：新旧页面都能编译，JSON 能输出新增结构。

### Phase 5：powermetrics 高级采集

1. 新建 `src/collectors/powermetrics.rs`。
2. 从 `cli.rs` 迁移 powermetrics 命令调用逻辑。
3. 建立 `PowermetricsSnapshot` 中间结构，先解析 CPU/GPU/ANE/DRAM/package power。
4. 加 timeout、错误缓存、最近成功数据缓存。
5. UI 显示采集模式：`Basic` / `Enhanced` / `SudoPower`。

验收：不 sudo 能跑，sudo 能显示高级功耗，powermetrics 卡住不会卡 UI。

### Phase 5.5：Terminal / PTY 轻量采集

1. 把 `src/collectors/terminal.rs` 从阻塞 `std::process::Command` 迁移到 `tokio::process::Command`。
2. 新增 `read_ptmx_max()`，读取 `sysctl -n kern.tty.ptmx_max`，缺失时返回 `None`。
3. 新增 `read_kern_tty_sysctls()`，尝试读取 `sysctl kern.tty`，没有输出时返回空数组。
4. 新增 session 采集：`who` / `w -h`，解析 local/remote session。
5. 将 `lsof` 改成低频慢路径，默认 30 秒执行一次，只用于 top PTY processes。
6. 给 `TerminalCollector` 加 `Arc<RwLock<TerminalInfo>>` 缓存，UI 只读缓存。
7. 在 Overview 和 IO 页展示 PTY 摘要、PTY pressure 和 top process。

验收：PTY 信息能显示，`kern.tty.ptmx_max` 不存在也不报错，`lsof` 不会每秒执行，UI 不被外部命令卡住。

### Phase 6：深挖页面

1. 实现 `cpu.rs`：P/E core、频率、residency、历史曲线。
2. 实现 `gpu.rs`：GPU/ANE 使用率、功耗、历史。
3. 实现 `memory.rs`：RAM、swap、pressure、DRAM。
4. 实现 `process.rs`：滚动、排序、过滤。
5. 实现 `io.rs`：Disk、Network、Thunderbolt。
6. 实现 `health.rs`：Thermal、Fan、Sensors。

验收：每个页面有独立价值，不是 Overview 的重复排列。

### Phase 7：打磨与发布

1. README 更新为 mactop++ 定位，加页面说明、快捷键、sudo 说明。
2. 录制 GIF 或更新 `images` 截图。
3. 增加 release build 检查。
4. 增加 smoke test：`--json`、`--csv`、`--stream json` 不破坏。
5. 准备改名策略：保留 `system-alert`，提供 `mactop-rs` 或 `rtop` alias。

验收：文档、截图、功能、导出、TUI 都可演示。

## 10. 组件实现规范

### 10.1 Metric Card

- 标题大写，例如 `CPU`、`GPU`、`MEMORY`。
- 第一行放主数字，例如 `74%`、`21.4W`。
- 第二行放趋势或补充，例如 `P 82% / E 31%`。
- 边框颜色由指标类型决定。

### 10.2 Core Matrix

- P-core 使用 `cpu_p`，E-core 使用 `cpu_e`。
- 每个核心块格式：`P0 █████░ 82% 3200MHz`。
- 小屏只显示：`P0 ████ 82%`。
- 缺频率显示 `----MHz` 或隐藏频率列。

### 10.3 Power Stack

- 展示 CPU/GPU/ANE/DRAM/package。
- package 是总值，其他是组成值。
- 如果没有 package，则显示各部分相加估算值并标注 `est`。

### 10.4 Process Table

- 表头固定，内容可滚动。
- 当前排序列高亮。
- CPU 超过 50% 用 warning，超过 100% 或异常用 critical。
- 进程名过长要截断，不能撑破布局。

### 10.5 Status Bar

- 左侧显示页面快捷键。
- 中间显示采集模式和刷新状态。
- 右侧显示主题名、通知状态、退出键。
- 示例：`1 overview 2 cpu 3 gpu 4 mem 5 proc | SudoPower 1s | theme mactop_pro | q quit`。

## 11. 验收清单

- 默认启动进入 Overview，不再是普通 full layout。
- `Tab` 和数字键可以分页。
- 至少四套主题可切换，默认 `mactop_pro`。
- sudo 和非 sudo 都能运行，非 sudo 不崩溃。
- 窗口缩小时自动 compact/minimal。
- 首页必须展示 CPU、P/E core、GPU、ANE、Memory、Power、Process、Network/Disk。
- 首页必须展示高密度 Terminal / PTY 摘要，例如 active/max、pressure、local/remote sessions。
- Process 页支持排序和滚动。
- IO 页必须展示 `kern.tty.ptmx_max`、PTY 使用率、top PTY processes；字段不可用时显示 `--` 而不是隐藏整个面板。
- API / JSON / CSV 不因为新增字段崩溃。
- README 有截图、快捷键、sudo 说明。

## 12. 推荐优先级

最先做这三件事，收益最大：

1. `Overview` 首页 + 分页快捷键。
2. `mactop_pro` / `cyberpunk_plus` 主题 token。
3. `PowermetricsCollector` 独立化和缓存化。
4. `TerminalCollector` async 化，补齐 `kern.tty.ptmx_max`、PTY pressure、session 摘要。

完成这三项后，项目观感会立刻从普通系统监控器升级成 mactop++ 雏形。
