/// English translations for all UI strings
pub fn translations() -> Vec<((&'static str, &'static str), &'static str)> {
    vec![
        // Header labels
        (("en", "cpu"), "CPU"),
        (("en", "memory"), "MEMORY"),
        (("en", "battery"), "BATTERY"),
        (("en", "power"), "POWER"),
        (("en", "network"), "NETWORK"),
        (("en", "thermal"), "THERMAL"),
        (("en", "processes"), "PROCESSES"),
        (("en", "cpu_cores"), "CPU CORES"),
        (("en", "cpu_history"), "CPU HISTORY"),
        (("en", "net_rx"), "NET RX"),
        // Status labels
        (("en", "health"), "Health"),
        (("en", "cycles"), "Cycles"),
        (("en", "pressure"), "Pressure"),
        (("en", "throttle"), "Throttle"),
        (("en", "fans"), "Fans"),
        (("en", "total"), "Total"),
        (("en", "loading"), "Loading..."),
        // Notification messages
        (("en", "cpu_critical"), "CPU usage critical"),
        (("en", "memory_critical"), "Memory usage critical"),
        (("en", "temperature_critical"), "Temperature critical"),
        (("en", "cpu_warning"), "CPU usage warning"),
        (("en", "memory_warning"), "Memory usage warning"),
        (("en", "temperature_warning"), "Temperature warning"),
        // Loading screen
        (("en", "loading_quantum"), "Initializing quantum sensors..."),
        (("en", "loading_neural"), "Calibrating neural network..."),
        (("en", "loading_particle"), "Loading particle accelerators..."),
        (("en", "loading_abort"), "Press 'q' to abort"),
        // Achievements
        (("en", "achievements_title"), "Achievements Unlocked"),
        (("en", "achievement_ice_cold"), "Ice Cold"),
        (("en", "achievement_power_saver"), "Power Saver"),
        (("en", "achievement_marathon"), "Marathon"),
        (("en", "achievement_fire_hazard"), "Fire Hazard"),
        // Predictions
        (("en", "battery_estimated"), "Battery estimated"),
        (("en", "memory_trend"), "Memory trend"),
        (("en", "cpu_trend_rising"), "CPU trend rising"),
        (("en", "anomaly_detected"), "anomaly detected"),
        // General
        (("en", "quit"), "Quit"),
        (("en", "refresh"), "Refresh"),
        (("en", "theme"), "Theme"),
        (("en", "notifications"), "Notifications"),
        (("en", "enabled"), "enabled"),
        (("en", "disabled"), "disabled"),
        (("en", "yes"), "YES"),
        (("en", "no"), "NO"),
        // Time travel
        (("en", "time_travel_mode"), "Time Travel Mode"),
        (("en", "time_travel_exit"), "Press ESC to exit"),
        (("en", "time_travel_navigate"), "Left/Right to navigate"),
        // Carbon
        (("en", "co2_today"), "CO2 today"),
        (("en", "energy_consumed"), "Energy consumed"),
        // API mode
        (("en", "server_started"), "Server started"),
        (("en", "metrics_available"), "Metrics available at"),
    ]
}
