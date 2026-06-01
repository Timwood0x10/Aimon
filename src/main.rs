use aimon::{
    cli::{check_root, handle_input, parse_args, InputEvent},
    collectors::DataCollector,
    config::{Config, ProcessSortBy},
    history::HistoryData,
    notification::NotificationManager,
    types::*,
    ui::layouts::LayoutType,
    ui::UI,
};

use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Advanced System Monitor starting...");

    // Check root privileges (but don't exit if not root, just warn)
    if let Err(e) = check_root().await {
        warn!("Running without root privileges: {}", e);
        warn!("Some features (like powermetrics) may not work properly.");
        warn!("For full functionality, run with: sudo cargo run");
    }

    // Parse command line arguments
    let cli_args = parse_args();

    // Load configuration
    let mut config = if let Some(config_file) = &cli_args.config_file {
        match Config::load_from_file(config_file) {
            Ok(config) => {
                info!("Loaded configuration from: {}", config_file);
                config
            }
            Err(e) => {
                warn!(
                    "Failed to load config file {}: {}. Using defaults.",
                    config_file, e
                );
                Config::default()
            }
        }
    } else {
        Config::default()
    };

    // Apply CLI overrides
    config.merge_with_cli(
        cli_args.refresh_rate,
        cli_args.minimal_mode,
        cli_args.theme.as_deref(),
    );
    config.merge_fan_control(cli_args.allow_fan_control);

    // Handle headless/API modes before UI initialization
    if cli_args.json_output {
        let mut exporter = aimon::api::HeadlessExporter::new();
        match exporter.export_json().await {
            Ok(output) => {
                println!("{}", output);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error collecting data: {}", e);
                std::process::exit(1);
            }
        }
    }

    if cli_args.csv_output {
        let mut exporter = aimon::api::HeadlessExporter::new();
        match exporter.export_csv().await {
            Ok(output) => {
                print!("{}", output);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error collecting data: {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref format_str) = cli_args.stream_format {
        match aimon::api::OutputFormat::parse_str(format_str) {
            Some(format) => {
                let interval_secs = cli_args.refresh_rate.unwrap_or(1);
                let interval_ms = cli_args.refresh_rate_ms.unwrap_or(0);
                let interval_val = if interval_ms > 0 {
                    interval_ms
                } else {
                    interval_secs * 1000
                };
                if let Err(e) =
                    aimon::api::HeadlessExporter::export_stream(interval_val, format).await
                {
                    eprintln!("Stream error: {}", e);
                    std::process::exit(1);
                }
                return Ok(());
            }
            None => {
                eprintln!(
                    "Unknown stream format: {}. Use json, csv, or prometheus.",
                    format_str
                );
                std::process::exit(1);
            }
        }
    }

    // ── UI init ────────────────────────────────────────────────────────
    let mut ui = UI::with_theme(&config.display.theme)?;
    info!(
        "UI initialized with theme '{}' - starting data collection in background...",
        config.display.theme
    );
    ui.show_loading_screen()?;

    // ── Shared state: data cache for background collector → UI reader ──
    // mactop pattern: collector writes to shared state, renderer reads from it.
    // Decoupled via Arc<RwLock>, no channel needed for data itself.
    let system_data: Arc<RwLock<SystemData>> = Arc::new(RwLock::new(create_placeholder_data()));
    let history: Arc<RwLock<HistoryData>> =
        Arc::new(RwLock::new(HistoryData::new(config.display.history_size)));

    // Flag to signal that new data is available for rendering
    let data_updated: Arc<AtomicBool> = Arc::new(AtomicBool::new(true)); // true for first frame

    // ── Background data collection thread ──────────────────────────────
    // Runs on its own schedule, never blocks the input/render path.
    // Uses std::thread to avoid Box<dyn Error> Send requirement from tokio::spawn.
    let _collector_thread = {
        let system_data = Arc::clone(&system_data);
        let history = Arc::clone(&history);
        let data_updated = Arc::clone(&data_updated);
        let runtime_handle = tokio::runtime::Handle::current();
        let refresh_ms = if config.refresh_rate_ms > 0 {
            config.refresh_rate_ms
        } else {
            config.refresh_rate * 1000
        };
        let refresh_duration = Duration::from_millis(refresh_ms);
        let fan_control = config.fan_control.clone();

        std::thread::spawn(move || {
            runtime_handle.block_on(async move {
                let mut collector = DataCollector::new_fast().with_fan_control(fan_control);
                let mut tick = interval(refresh_duration);
                loop {
                    match collector.collect_all_data().await {
                        Ok(new_data) => {
                            {
                                let mut cache = system_data.write().await;
                                *cache = new_data;
                            }
                            {
                                let data_snap = system_data.read().await;
                                let mut hist = history.write().await;
                                hist.update_from_system_data(&data_snap);
                            }
                            data_updated.store(true, Ordering::Relaxed);
                        }
                        Err(e) => {
                            error!("Data collection error: {}", e);
                        }
                    }
                    tick.tick().await;
                }
            })
        })
    };

    // Notification manager runs in main loop (avoids Send bound issues)
    let mut notification_manager = NotificationManager::new(
        config.notifications.enabled,
        config.notifications.cooldown_seconds,
    );
    let mut last_notif_check = std::time::Instant::now();

    // ── Input handling (already on std::thread via handle_input) ───────
    let mut input_receiver = handle_input().await;

    // ── Render timer: independent of data collection ───────────────────
    // mactop pattern: ticker-driven render at refresh rate.
    // Redraws when: (1) data_updated flag from collector, (2) user input.
    let mut render_interval = interval(Duration::from_millis(33)); // ~30 FPS

    info!("System monitor initialized. Press '?' for help, 'q' to quit.");

    // ── Main event loop ────────────────────────────────────────────────
    // mactop pattern: select on input + ticker.
    // Input → immediate dirty flag (instant redraw like mactop's drawScreen).
    // Render → draw when dirty or data changed, never blocks on collection.
    // Collector → background thread, writes to Arc<RwLock>.
    loop {
        tokio::select! {
            // ── Input branch: process event, mark dirty for instant redraw ──
            input_event = input_receiver.recv() => {
                let handled = match input_event {
                    Some(InputEvent::Quit) => {
                        if ui.is_session_report_visible() {
                            info!("Quit signal received");
                            break;
                        }
                        ui.toggle_session_report();
                        true
                    }
                    Some(InputEvent::NextTab) | Some(InputEvent::NextLayout) => {
                        ui.next_layout();
                        true
                    }
                    Some(InputEvent::PreviousTab) | Some(InputEvent::PreviousLayout) => {
                        ui.previous_layout();
                        true
                    }
                    Some(InputEvent::ScrollUp) => {
                        ui.scroll_up();
                        true
                    }
                    Some(InputEvent::ScrollDown) => {
                        ui.scroll_down();
                        true
                    }
                    Some(InputEvent::GoToTop) => {
                        ui.go_to_top();
                        true
                    }
                    Some(InputEvent::GoToBottom) => {
                        ui.go_to_bottom();
                        true
                    }
                    Some(InputEvent::JumpToLayout(n)) => {
                        if let Some(layout) = LayoutType::from_key_number(n) {
                            ui.set_layout(layout);
                        }
                        true
                    }
                    Some(InputEvent::CycleSortForward) => {
                        config.process_sort_by = match config.process_sort_by {
                            ProcessSortBy::Cpu => ProcessSortBy::Memory,
                            ProcessSortBy::Memory => ProcessSortBy::Pid,
                            ProcessSortBy::Pid => ProcessSortBy::Name,
                            ProcessSortBy::Name => ProcessSortBy::Cpu,
                        };
                        info!("Sort order: {:?}", config.process_sort_by);
                        true
                    }
                    Some(InputEvent::CycleSortBackward) => {
                        config.process_sort_by = match config.process_sort_by {
                            ProcessSortBy::Cpu => ProcessSortBy::Name,
                            ProcessSortBy::Memory => ProcessSortBy::Cpu,
                            ProcessSortBy::Pid => ProcessSortBy::Memory,
                            ProcessSortBy::Name => ProcessSortBy::Pid,
                        };
                        info!("Sort order: {:?}", config.process_sort_by);
                        true
                    }
                    Some(InputEvent::ToggleHelp) => {
                        ui.toggle_help();
                        true
                    }
                    Some(InputEvent::ToggleNotifications) => {
                        let new_state = !config.notifications.enabled;
                        config.notifications.enabled = new_state;
                        notification_manager.set_enabled(new_state);
                        info!("Notifications {}", if new_state { "enabled" } else { "disabled" });
                        true
                    }
                    Some(InputEvent::Refresh) => {
                        true // Force dirty for immediate redraw
                    }
                    Some(InputEvent::ShowSessionReport) => {
                        ui.toggle_session_report();
                        true
                    }
                    Some(InputEvent::CycleTheme) => {
                        let themes = aimon::ui::theme::Theme::all_themes();
                        let current_index = themes.iter().position(|&t| t == config.display.theme).unwrap_or(0);
                        let next_index = (current_index + 1) % themes.len();
                        config.display.theme = themes[next_index].to_string();
                        ui.set_theme(&config.display.theme);
                        info!("Theme changed to: {}", config.display.theme);
                        true
                    }
                    Some(InputEvent::TogglePartyMode) => {
                        ui.toggle_party_mode();
                        true
                    }
                    Some(InputEvent::SearchProcess) => {
                        info!("Search not yet implemented");
                        false
                    }
                    Some(InputEvent::KillProcess) => {
                        info!("Kill process not yet implemented");
                        false
                    }
                    Some(InputEvent::ToggleTimeTravel) => {
                        info!("Time travel not yet implemented");
                        false
                    }
                    Some(InputEvent::ToggleAchievements) => {
                        info!("Achievements not yet implemented");
                        false
                    }
                    None => {
                        warn!("Input channel closed");
                        break;
                    }
                };

                if handled {
                    // mactop pattern: immediate redraw after every key event
                    let data_snap = system_data.read().await;
                    let hist_snap = history.read().await;
                    if let Err(e) = ui.draw(&data_snap, &hist_snap, &config) {
                        error!("UI draw error: {}", e);
                    }
                }
            }

            // ── Render branch: draw at ~30 FPS when new data available ────
            // mactop pattern: ticker-driven render when collector wrote new data
            _ = render_interval.tick(), if data_updated.load(Ordering::Relaxed) => {
                let data_snap = system_data.read().await;
                let hist_snap = history.read().await;
                if let Err(e) = ui.draw(&data_snap, &hist_snap, &config) {
                    error!("UI draw error: {}", e);
                }
                data_updated.store(false, Ordering::Relaxed);

                // Low-frequency notification check (every ~5s)
                if last_notif_check.elapsed() >= Duration::from_secs(5) {
                    last_notif_check = std::time::Instant::now();
                    if let Err(e) = notification_manager
                        .check_and_send_notifications(&data_snap, &config.thresholds)
                        .await
                    {
                        error!("Notification error: {}", e);
                    }
                }
            }
        }
    }

    // Save runtime state before exit
    ui.save_runtime_state(&config);

    // Cleanup — collector thread will die when the process exits
    ui.cleanup()?;
    info!("System monitor exited normally");

    Ok(())
}

fn create_placeholder_data() -> SystemData {
    use std::time::Instant;

    SystemData {
        system_info: SystemInfo {
            name: "macOS".to_string(),
            kernel_version: "Loading...".to_string(),
            os_version: "Loading...".to_string(),
            host_name: "Loading...".to_string(),
            cpu_arch: "arm64".to_string(),
            cpu_brand: "Apple Silicon".to_string(),
            cpu_core_count: 0,
            e_core_count: 0,
            p_core_count: 0,
            gpu_core_count: 0,
            chip_name: "Loading...".to_string(),
        },
        cpu_info: CpuInfo {
            core_usages: vec![0.0; 8],
            average_usage: 0.0,
            power_metrics: CPUMetrics::default(),
            usage_meta: MetricMeta::unavailable(),
            power_meta: MetricMeta::unavailable(),
        },
        gpu_info: GpuInfo::default(),
        ane_info: AneInfo::default(),
        dram_info: DramInfo::default(),
        thunderbolt_info: ThunderboltInfo::default(),
        disk_io_info: DiskIoInfo::default(),
        disk_usage_info: Vec::new(),
        directory_usage_info: Vec::new(),
        memory_info: MemoryInfo {
            total_memory: 0,
            used_memory: 0,
            available_memory: 0,
            total_swap: 0,
            used_swap: 0,
            usage_percentage: 0,
        },
        network_info: vec![],
        temperature_info: vec![],
        process_info: vec![],
        battery_info: BatteryInfo::default(),
        thermal_info: ThermalInfo::default(),
        performance_metrics: PerformanceMetrics::default(),
        system_health: SystemHealthInfo::default(),
        terminal_info: TerminalInfo::default(),
        carbon_info: aimon::carbon::tracker::CarbonTracker::default(),
        capabilities: aimon::collectors::capabilities::CollectorCapabilities::default(),
        timestamp: Instant::now(),
    }
}
