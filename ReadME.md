# System Alert - Advanced macOS System Monitor

A high-performance, feature-rich system monitoring tool for macOS with special optimizations for Apple Silicon chips. This tool provides real-time system metrics with an intuitive terminal user interface, smart notifications, and comprehensive configuration options.

## 🎯 Overview

System Alert is a professional-grade system monitoring application designed specifically for macOS, with enhanced support for Apple Silicon processors. It offers real-time monitoring of CPU, memory, battery, temperature, network, and power consumption with a beautiful terminal-based interface.

### 🌟 Highlights

- **🔋 Advanced Battery Monitoring**: Real-time battery health, cycle count, and charging status
- **⚡ Apple Silicon Optimization**: E-cluster/P-cluster monitoring with detailed power metrics
- **🎨 Beautiful TUI Interface**: Clean, organized four-quadrant layout with progress bars
- **📊 Real-time Data**: Live system metrics with configurable refresh rates
- **🔔 Smart Notifications**: Configurable threshold-based alerts
- **⚙️ Highly Configurable**: TOML-based configuration with CLI overrides

## 🚀 Key Features & Optimizations

### Performance Improvements
- **Async Architecture**: Non-blocking data collection using Tokio
- **Smart Caching**: PowerMetrics data cached for optimal performance
- **Efficient Memory Management**: Reduced allocations and optimized data structures
- **Selective Refresh**: Only refresh necessary system components
- **Optimized Build**: Release profile with LTO, size optimization, and panic=abort

### Enhanced User Experience
- **Four-Quadrant Layout**: Brand new sectioned layout with clear information grouping
- **Interactive Controls**: Keyboard navigation, notification toggle, manual refresh
- **Smart Notifications**: Configurable thresholds with cooldown periods
- **Visual Power Monitor**: Beautiful bordered display with progress bars
- **Minimal Mode**: Lightweight display for resource-constrained scenarios

### Advanced Features
- **Configuration Management**: TOML-based config files with CLI overrides
- **Apple Silicon Metrics**: E-cluster/P-cluster monitoring with power analysis
- **Temperature Monitoring**: Component temperature tracking with smart status indicators
- **Process Analysis**: Top processes by CPU usage with detailed information
- **Network Statistics**: Real-time network traffic monitoring
- **Real-time Power Statistics**: Dedicated power consumption analysis

## 📋 System Requirements

- **macOS**: 10.15+ (Optimized for Apple Silicon)
- **Root Privileges**: Required for accessing system metrics via `powermetrics`
- **Rust**: 1.70+ (Required for building from source)

## 🛠 Installation

### Install from Source
```bash
git clone https://github.com/yourusername/system-alert.git
cd system-alert
cargo build --release
```

### Quick Start
```bash
# Run with default settings
sudo cargo run

# Run with custom refresh rate
sudo cargo run -- --refresh 2

# Run in minimal mode
sudo cargo run -- --minimal

# Run with custom config
sudo cargo run -- --config custom-config.toml
```

![image](images/ui.png)

## 🎮 Usage & Controls

### Command Line Options
```bash
sudo ./target/release/system-alert [options]

Options:
  -r, --refresh <seconds>     Set refresh rate (default: 1)
  -m, --minimal              Use minimal display mode
  -c, --config <file>        Specify custom config file
  -h, --help                 Show help message
  -V, --version              Show version information
```

### Interactive Controls
- **q** or **Ctrl+C**: Quit application
- **n**: Toggle notifications
- **r**: Force refresh
- **t**: Cycle through themes

### Interface Layout

The application features a modern four-quadrant layout:

1. **🔵 CPU Section** (Top Left): 
   - CPU core usage with individual core monitoring
   - Apple Silicon E-cluster/P-cluster metrics
   - Real-time frequency and power consumption

2. **🔴 Power Monitor** (Top Right):
   - Battery status with health percentage
   - Charging state and time remaining
   - Power adapter wattage and cycle count
   - Comprehensive power analytics

3. **🟢 Memory Monitor** (Bottom Left):
   - RAM usage with detailed breakdown
   - Swap memory statistics
   - Memory pressure indicators

4. **🟡 Temperature Monitor** (Bottom Left):
   - Component temperature readings
   - Thermal throttling status
   - Fan speed monitoring

5. **🟣 Process Monitor** (Bottom Right):
   - Top processes by CPU usage
   - Memory consumption per process
   - Real-time process statistics

6. **🔵 Network Monitor** (Bottom Right):
   - Network interface statistics
   - Bytes transmitted/received
   - Packet counts and rates

## 🎨 Themes

System Alert comes with 9 built-in themes to customize your monitoring experience:

| Theme | Description | Style |
|-------|-------------|-------|
| **cyberpunk** | Neon-inspired dark theme (default) | Futuristic, vibrant |
| **nord** | Nordic minimalist theme | Cool, calm, professional |
| **dracula** | Classic dark theme | Vibrant, easy on eyes |
| **tokyo_night** | Tokyo night sky inspired | Modern, sleek |
| **monokai** | Classic editor theme | Warm, familiar |
| **solarized_dark** | Solarized dark variant | Balanced, scientific |
| **gruvbox** | Retro-inspired theme | Warm, nostalgic |
| **catppuccin** | Soft pastel theme | Gentle, modern |
| **one_dark** | Atom editor theme | Clean, professional |

### Using Themes

**Via configuration file:**
```toml
[display]
theme = "nord"
```

**Via command line:**
```bash
cargo run -- --theme dracula
```

**Interactive switching:**
Press `t` to cycle through all available themes while the application is running.

## ⚙️ Configuration Options

### Default Configuration
The application will create a `config.toml` file with sensible defaults:

```toml
# System Monitor Configuration
refresh_rate = 1
minimal_mode = false

[thresholds]
cpu_warning = 75.0
cpu_critical = 90.0
memory_warning = 75
memory_critical = 90
temperature_warning = 70.0
temperature_critical = 85.0

[display]
show_temperatures = true
show_network = true
show_processes = true
show_history = true
history_size = 60

[notifications]
enabled = true
cooldown_seconds = 30
```

### Configuration Options

#### Main Settings
- `refresh_rate`: Update interval in seconds
- `minimal_mode`: Enable simplified interface

#### Threshold Settings
- `cpu_warning/critical`: CPU usage alert thresholds (%)
- `memory_warning/critical`: Memory usage alert thresholds (%)
- `temperature_warning/critical`: Temperature alert thresholds (°C)

#### Display Settings
- `show_temperatures`: Enable temperature monitoring
- `show_network`: Enable network interface monitoring
- `show_processes`: Enable process monitoring
- `show_history`: Enable historical data tracking
- `history_size`: Number of data points to retain

#### Notification Settings
- `enabled`: Enable/disable system notifications
- `cooldown_seconds`: Minimum time between notifications

## 🏗 Architecture Overview

Optimized architecture with separation of concerns for better maintainability and performance:

```
src/
├── main.rs              # Application entry point with async event loop
├── cli.rs               # Command line parsing and input handling
├── config.rs            # Configuration management (TOML)
├── data_collector.rs    # Async system data collection
├── battery_collector.rs # Advanced battery data collection
├── ui.rs                # Terminal user interface (TUI)
├── notification.rs      # Smart notification system
├── history.rs           # Historical data tracking
├── types.rs             # Data structures and types
└── system_info.rs       # Legacy compatibility module
```

### Key Improvements
1. **Separation of Concerns**: Clear separation between UI, data collection, and business logic
2. **Async/Await**: Non-blocking operations throughout
3. **Error Handling**: Comprehensive error handling without panics
4. **Memory Efficiency**: Reduced allocations and better data structures
5. **Configurability**: Extensive configuration options
6. **Extensibility**: Modular design for easy feature addition

## 🔧 Performance Optimization

### Build Optimization
```toml
[profile.release]
strip = true           # Remove debug symbols
opt-level = "z"        # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Single codegen unit for better optimization
panic = "abort"       # Smaller binary size
```

### Runtime Optimization
- **PowerMetrics Caching**: Reduce expensive system calls
- **Selective Data Refresh**: Only update changed components
- **Efficient String Handling**: Minimize allocations in hot paths
- **Smart Rendering**: Only redraw when necessary
- **Async I/O**: Non-blocking system calls

## 🚨 Troubleshooting

### Common Issues

**Permission Denied**
```bash
# Make sure to run with sudo
sudo cargo run
```

**PowerMetrics Not Found**
```bash
# Verify powermetrics is available (macOS only)
which powermetrics
```

**High CPU Usage**
```bash
# Increase refresh rate to reduce system load
sudo cargo run -- --refresh 5
```

**Configuration Issues**
```bash
# Reset to defaults by removing config file
rm config.toml
sudo cargo run
```

## 📊 Apple Silicon Metrics

For Apple Silicon Macs, this tool provides detailed metrics:

- **E-Cluster**: Efficiency core usage and frequency
- **P-Cluster**: Performance core usage and frequency
- **Power Consumption**: CPU, GPU, ANE (Neural Engine), and total package power
- **Real-time Monitoring**: Live updates of power states

## 🤝 Contributing

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests.

### Development Setup
```bash
git clone https://github.com/yourusername/system-alert.git
cd system-alert
cargo build
cargo test
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [TUI-rs](https://github.com/fdehau/tui-rs) for terminal interface
- Leverages [sysinfo](https://github.com/GuillaumeGomez/sysinfo) for system metrics
- Powered by [Tokio](https://tokio.rs/) for async runtime

