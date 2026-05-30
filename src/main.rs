use system_alert::{
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
use std::time::Duration;
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

    // Handle headless/API modes before UI initialization
    if cli_args.json_output {
        let mut exporter = system_alert::api::HeadlessExporter::new();
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
        let mut exporter = system_alert::api::HeadlessExporter::new();
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
        match system_alert::api::OutputFormat::parse_str(format_str) {
            Some(format) => {
                let interval = cli_args.refresh_rate.unwrap_or(1);
                if let Err(e) =
                    system_alert::api::HeadlessExporter::export_stream(interval, format).await
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

    // Shared data collector for API server mode (held alive for server lifetime)
    let _api_collector = if cli_args.server_mode {
        let collector = std::sync::Arc::new(tokio::sync::Mutex::new(
            system_alert::collectors::DataCollector::new_fast(),
        ));
        if let Err(e) =
            system_alert::api::ApiServer::start("0.0.0.0", cli_args.port, collector.clone()).await
        {
            eprintln!("Failed to start API server: {}", e);
            std::process::exit(1);
        }
        Some(collector)
    } else {
        None
    };

    // Initialize UI first - immediate startup
    let mut ui = UI::with_theme(&config.display.theme)?;
    info!(
        "UI initialized with theme '{}' - starting data collection in background...",
        config.display.theme
    );

    // Show loading screen immediately
    ui.show_loading_screen()?;

    // Initialize other components in background
    let mut data_collector = DataCollector::new_fast();
    let mut history = HistoryData::new(config.display.history_size);
    let mut notification_manager = NotificationManager::new(
        config.notifications.enabled,
        config.notifications.cooldown_seconds,
    );

    // Set up input handling
    let mut input_receiver = handle_input().await;

    // Set up refresh timer
    let mut refresh_interval = interval(Duration::from_secs(config.refresh_rate));

    // Create initial empty data for immediate display
    let mut system_data = create_placeholder_data();

    info!("System monitor initialized. Press '?' for help, 'q' to quit.");

    // Main event loop
    loop {
        tokio::select! {
            // Handle input events
            input_event = input_receiver.recv() => {
                match input_event {
                    Some(InputEvent::Quit) => {
                        info!("Quit signal received");
                        break;
                    }
                    Some(InputEvent::NextTab) => {
                        ui.next_layout();
                    }
                    Some(InputEvent::PreviousTab) => {
                        ui.previous_layout();
                    }
                    Some(InputEvent::NextLayout) => {
                        ui.next_layout();
                    }
                    Some(InputEvent::PreviousLayout) => {
                        ui.previous_layout();
                    }
                    Some(InputEvent::ScrollUp) => {
                        ui.scroll_up();
                    }
                    Some(InputEvent::ScrollDown) => {
                        ui.scroll_down();
                    }
                    Some(InputEvent::GoToTop) => {
                        ui.go_to_top();
                    }
                    Some(InputEvent::GoToBottom) => {
                        ui.go_to_bottom();
                    }
                    Some(InputEvent::JumpToLayout(n)) => {
                        if let Some(layout) = LayoutType::from_key_number(n) {
                            ui.set_layout(layout);
                        }
                    }
                    Some(InputEvent::CycleSortForward) => {
                        config.process_sort_by = match config.process_sort_by {
                            ProcessSortBy::Cpu => ProcessSortBy::Memory,
                            ProcessSortBy::Memory => ProcessSortBy::Pid,
                            ProcessSortBy::Pid => ProcessSortBy::Name,
                            ProcessSortBy::Name => ProcessSortBy::Cpu,
                        };
                        info!("Sort order: {:?}", config.process_sort_by);
                    }
                    Some(InputEvent::CycleSortBackward) => {
                        config.process_sort_by = match config.process_sort_by {
                            ProcessSortBy::Cpu => ProcessSortBy::Name,
                            ProcessSortBy::Memory => ProcessSortBy::Cpu,
                            ProcessSortBy::Pid => ProcessSortBy::Memory,
                            ProcessSortBy::Name => ProcessSortBy::Pid,
                        };
                        info!("Sort order: {:?}", config.process_sort_by);
                    }
                    Some(InputEvent::ToggleHelp) => {
                        ui.toggle_help();
                    }
                    Some(InputEvent::ToggleNotifications) => {
                        let new_state = !config.notifications.enabled;
                        config.notifications.enabled = new_state;
                        notification_manager.set_enabled(new_state);
                        info!("Notifications {}", if new_state { "enabled" } else { "disabled" });
                    }
                    Some(InputEvent::Refresh) => {
                        // Force immediate refresh by continuing to the refresh logic
                    }
                    Some(InputEvent::CycleTheme) => {
                        let themes = system_alert::ui::theme::Theme::all_themes();
                        let current_index = themes.iter().position(|&t| t == config.display.theme).unwrap_or(0);
                        let next_index = (current_index + 1) % themes.len();
                        config.display.theme = themes[next_index].to_string();

                        // Recreate UI with new theme
                        ui = UI::with_theme_and_layout(&config.display.theme, ui.current_layout())?;
                        info!("Theme changed to: {}", config.display.theme);
                    }
                    Some(InputEvent::TogglePartyMode) => {
                        ui.toggle_party_mode();
                    }
                    Some(InputEvent::SearchProcess) => {
                        // Search not yet implemented - placeholder event
                        info!("Search not yet implemented");
                    }
                    Some(InputEvent::KillProcess) => {
                        // Kill not yet implemented - placeholder event
                        info!("Kill process not yet implemented");
                    }
                    Some(InputEvent::ToggleTimeTravel) => {
                        info!("Time travel not yet implemented");
                    }
                    Some(InputEvent::ToggleAchievements) => {
                        info!("Achievements not yet implemented");
                    }
                    None => {
                        warn!("Input channel closed");
                        break;
                    }
                }
            }

            // Handle periodic refresh
            _ = refresh_interval.tick() => {
                // Collect system data asynchronously
                match data_collector.collect_all_data().await {
                    Ok(new_data) => {
                        system_data = new_data;
                        // Update history
                        history.update_from_system_data(&system_data);

                        // Check for notifications
                        if let Err(e) = notification_manager
                            .check_and_send_notifications(&system_data, &config.thresholds)
                            .await
                        {
                            error!("Notification error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Data collection error: {}", e);
                        // Keep using previous data, don't crash
                    }
                }

                // Always update UI (even with old data)
                if let Err(e) = ui.draw(&system_data, &history, &config) {
                    error!("UI draw error: {}", e);
                }
            }
        }
    }

    // Save runtime state before exit
    ui.save_runtime_state(&config);

    // Cleanup
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
            core_usages: vec![0.0; 8], // 8 cores placeholder
            average_usage: 0.0,
            power_metrics: CPUMetrics::default(),
        },
        gpu_info: GpuInfo::default(),
        ane_info: AneInfo::default(),
        dram_info: DramInfo::default(),
        thunderbolt_info: ThunderboltInfo::default(),
        disk_io_info: DiskIoInfo::default(),
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
        timestamp: Instant::now(),
    }
}
