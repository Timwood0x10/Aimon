/// Chinese translations for all UI strings
pub fn translations() -> Vec<((&'static str, &'static str), &'static str)> {
    vec![
        // Header labels
        (("zh", "cpu"), "CPU"),
        (("zh", "memory"), "内存"),
        (("zh", "battery"), "电池"),
        (("zh", "power"), "功率"),
        (("zh", "network"), "网络"),
        (("zh", "thermal"), "温度"),
        (("zh", "processes"), "进程"),
        (("zh", "cpu_cores"), "CPU核心"),
        (("zh", "cpu_history"), "CPU历史"),
        (("zh", "net_rx"), "网络接收"),
        // Status labels
        (("zh", "health"), "健康"),
        (("zh", "cycles"), "循环次数"),
        (("zh", "pressure"), "压力"),
        (("zh", "throttle"), "降频"),
        (("zh", "fans"), "风扇"),
        (("zh", "total"), "总计"),
        (("zh", "loading"), "加载中..."),
        // Notification messages
        (("zh", "cpu_critical"), "CPU使用率严重过高"),
        (("zh", "memory_critical"), "内存使用率严重过高"),
        (("zh", "temperature_critical"), "温度严重过高"),
        (("zh", "cpu_warning"), "CPU使用率偏高"),
        (("zh", "memory_warning"), "内存使用率偏高"),
        (("zh", "temperature_warning"), "温度偏高"),
        // Loading screen
        (("zh", "loading_quantum"), "初始化量子传感器..."),
        (("zh", "loading_neural"), "校准神经网络..."),
        (("zh", "loading_particle"), "加载粒子加速器..."),
        (("zh", "loading_abort"), "按 'q' 退出"),
        // Achievements
        (("zh", "achievements_title"), "已解锁成就"),
        (("zh", "achievement_ice_cold"), "冰凉"),
        (("zh", "achievement_power_saver"), "省电达人"),
        (("zh", "achievement_marathon"), "马拉松"),
        (("zh", "achievement_fire_hazard"), "火炉"),
        // Predictions
        (("zh", "battery_estimated"), "电池预估"),
        (("zh", "memory_trend"), "内存趋势"),
        (("zh", "cpu_trend_rising"), "CPU趋势上升"),
        (("zh", "anomaly_detected"), "检测到异常"),
        // General
        (("zh", "quit"), "退出"),
        (("zh", "refresh"), "刷新"),
        (("zh", "theme"), "主题"),
        (("zh", "notifications"), "通知"),
        (("zh", "enabled"), "已启用"),
        (("zh", "disabled"), "已禁用"),
        (("zh", "yes"), "是"),
        (("zh", "no"), "否"),
        // Time travel
        (("zh", "time_travel_mode"), "时间旅行模式"),
        (("zh", "time_travel_exit"), "按 ESC 退出"),
        (("zh", "time_travel_navigate"), "左右键导航"),
        // Carbon
        (("zh", "co2_today"), "今日碳排放"),
        (("zh", "energy_consumed"), "能耗"),
        // API mode
        (("zh", "server_started"), "服务已启动"),
        (("zh", "metrics_available"), "指标访问地址"),
    ]
}
