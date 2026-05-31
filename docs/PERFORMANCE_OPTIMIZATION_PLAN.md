# Rust Mactop++ 性能优化方案

本文档解释当前项目按键响应慢的根因，并定义后续优化路线。核心结论：**Rust 完全可以比 Go 做得更快、更稳、更低延迟；当前项目慢不是 Rust 的问题，而是事件循环、采集任务和渲染路径没有解耦。**

## 1. 核心结论

- Rust 具备零成本抽象、无 GC 暂停、可控内存布局、低开销 async runtime，非常适合做高频 TUI。
- Go 的 mactop 响应快，主要来自架构：输入路径短、采集后台化、UI 立即重绘。
- 当前项目慢，是因为输入事件只修改状态，不立即重绘；慢采集还在主事件循环里 `await`。
- 优化方向不是“把刷新率调高”，而是拆成 `input task`、`render loop`、`collector task`、`slow command task`。
- 最终目标：按键到 UI 反馈 `< 16ms`，理想情况下 `< 8ms`。

## 2. 当前瓶颈

### 2.1 输入后没有立即重绘

当前主循环在 `src/main.rs:164` 收到输入后，只执行类似 `ui.next_layout()`、`ui.scroll_down()`、`ui.toggle_help()` 的状态修改。

真正重绘发生在 `src/main.rs:264` 的 refresh tick 分支：

```rust
_ = refresh_interval.tick() => {
    match data_collector.collect_all_data().await {
        Ok(new_data) => { ... }
        Err(e) => { ... }
    }

    if let Err(e) = ui.draw(&system_data, &history, &config) {
        error!("UI draw error: {}", e);
    }
}
```

这意味着按键后最坏要等一个完整刷新周期。如果刷新率是 1 秒，用户就会感觉“按键卡顿”。

### 2.2 慢采集阻塞主事件循环

`data_collector.collect_all_data().await` 发生在主 `tokio::select!` 的 refresh 分支里。只要采集没结束，主 loop 就不能继续处理输入。

典型慢源：

- `powermetrics`：外部命令，可能耗时数百毫秒到数秒。
- `ioreg` / `pmset` / `sysctl`：通常较快，但仍不应在热路径里阻塞。
- `lsof`：可能很慢，输出也可能很大。
- process/network/disk 枚举：数据量大时会造成瞬时压力。

### 2.3 渲染和采集频率绑定

当前 UI 重绘和数据刷新绑在一起。正确做法是：

- 输入来了：立即重绘。
- 数据来了：标记 dirty，然后重绘。
- 没有变化：不画，或者低频画时钟/动画。

数据刷新慢，不应该影响按键响应。

### 2.4 全屏重绘成本偏高

当前 `src/ui/mod.rs:159` 每次 draw 都完整渲染，并在 `src/ui/mod.rs:161` 全屏 fill/clear。全屏重绘不是不能做，但要控制频率，并避免和慢采集绑定。

### 2.5 主题切换重建 UI

`src/main.rs:229` 的主题切换会重新创建 `UI`：

```rust
ui = UI::with_theme_and_layout(&config.display.theme, ui.current_layout())?;
```

这比单纯替换 theme 对象更重。后续应改成 `ui.set_theme(...)`，避免重建 terminal backend。

## 3. 性能目标

| 指标 | 目标 | 说明 |
|---|---:|---|
| 按键到事件入队 | `< 1ms` | 输入线程只做解析和发送 |
| 按键到状态更新 | `< 2ms` | 主 loop 不等待采集 |
| 按键到重绘完成 | `< 8-16ms` | 60 FPS / 120 FPS 体感 |
| 普通指标刷新 | `500ms-1000ms` | CPU/mem/process/network |
| powermetrics 刷新 | `1000ms-2000ms` | sudo 高级指标 |
| terminal sysctl 刷新 | `5000ms` | `kern.tty.ptmx_max` 等轻量指标 |
| terminal lsof 刷新 | `10000ms-30000ms` | 慢路径，绝不进热路径 |
| 单次 draw 时间 | `< 4ms` | 复杂页面尽量不超过 8ms |
| 主线程阻塞 | `0` | 不在 UI loop 里执行外部命令 |

## 4. 目标架构

### 4.1 总体结构

```text
┌────────────────────┐
│ input thread/task   │  termion/crossterm key events
└─────────┬──────────┘
          │ UiEvent
          ▼
┌────────────────────┐       ┌────────────────────────┐
│ app/render loop     │◀──────│ watch<SystemData>       │
│ - update ui state   │       │ latest data snapshot    │
│ - mark dirty        │       └────────────────────────┘
│ - draw if needed    │
└─────────▲──────────┘       ┌────────────────────────┐
          │                  │ collector tasks         │
          └──────────────────│ basic/power/terminal/io │
                             └────────────────────────┘
```

原则：

- UI loop 永远不直接执行系统采集。
- UI loop 永远不等待 `powermetrics`、`lsof`、`ioreg`、`sysctl`。
- 所有采集结果写入快照缓存。
- 输入事件只改状态并触发 dirty。
- render tick 只负责“是否需要画”和“画”。

### 4.2 推荐 channel

```rust
pub enum AppEvent {
    Input(InputEvent),
    DataUpdated,
    Tick,
    Resize,
}

pub struct AppState {
    pub data: Arc<RwLock<SystemData>>,
    pub history: HistoryData,
    pub ui_state: UiState,
    pub config: Config,
    pub dirty: bool,
    pub paused: bool,
}
```

推荐通信方式：

- `mpsc::channel<AppEvent>`：输入、resize、tick 等事件。
- `watch::channel<SystemData>`：最新系统快照，只关心最新值。
- `Arc<RwLock<SystemData>>`：UI 读取最新数据，collector 写入。
- `CancellationToken`：退出时关闭所有后台 task。

## 5. 渲染循环设计

### 5.1 不要让 refresh tick 驱动一切

错误模式：

```rust
refresh_tick => collect_data().await; draw();
input_event => update_state(); // 不 draw
```

正确模式：

```rust
input_event => update_state(); dirty = true;
data_updated => update_history(); dirty = true;
render_tick => if dirty { draw(); dirty = false; }
```

### 5.2 render tick 建议

- 默认 `33ms`，约 30 FPS，足够流畅且省电。
- 如果启用动画或 party mode，可切到 `16ms`，约 60 FPS。
- 如果无变化，跳过 draw。
- 按键事件可以绕过 tick 立即 draw，也可以设置 dirty 等待下一个 16ms tick。

### 5.3 立即响应策略

最低成本第一刀：输入分支处理完后直接 draw。

```rust
Some(event) => {
    handle_input_event(event, &mut ui, &mut config);
    ui.draw(&system_data, &history, &config)?;
}
```

更完整方案：输入分支只设置 `dirty = true`，render tick 统一 draw。

```rust
Some(event) => {
    handle_input_event(event, &mut app_state);
    app_state.dirty = true;
}
```

## 6. 采集任务设计

### 6.1 BasicCollector Task

刷新频率：`500ms-1000ms`。

负责：

- CPU 使用率
- 内存
- 进程基础信息
- 网络速率
- 磁盘 IO
- uptime / load average

要求：

- 不执行慢外部命令。
- 每次采集超时控制在 `200ms-500ms`。
- 数据写入 `SystemData` 快照，不直接触发 UI draw，只发送 `DataUpdated`。

### 6.2 Powermetrics Task

刷新频率：`1000ms-2000ms`。

负责：

- CPU package power
- CPU/GPU/ANE/DRAM power
- P/E cluster frequency
- active residency
- thermal 相关高级指标

要求：

- 独立 task。
- 使用 timeout，建议 `1500ms-2500ms`。
- 失败时保留上一份成功数据。
- 未 sudo 时进入 disabled/locked 状态，不反复刷错误日志。

### 6.3 TerminalCollector Task

刷新频率：

- `sysctl -n kern.tty.ptmx_max`：`5s`
- `who` / `w -h`：`5s`
- `lsof`：`10s-30s`

负责：

- PTY active/max
- PTY pressure
- local/remote sessions
- top PTY processes
- `kern.tty.*` sysctl 快照

要求：

- 使用 `tokio::process::Command`。
- `lsof` 绝不每秒执行。
- `kern.tty.ptmx_max` 不存在时显示 `--`，不能报错刷屏。
- 采集结果进入缓存，UI 只读缓存。

### 6.4 ProcessCollector 优化

进程表很容易成为性能黑洞。

优化规则：

- 只在 collector task 排序一次，UI 不要每次 draw 都完整排序。
- 保留 top N，例如 top 100，UI 根据滚动 offset 展示其中一段。
- 进程名和命令行提前截断或缓存宽度。
- IO 字段低频更新，CPU/MEM 高频更新。

## 7. UI 渲染优化

### 7.1 组件分层

新组件要拆成可复用小 widget：

- `HeaderWidget`
- `MetricCard`
- `CoreMatrix`
- `PowerStack`
- `ProcessTable`
- `Sparkline`
- `StatusBar`

每个 widget 只拿自己需要的数据，避免 clone 大对象。

### 7.2 避免 draw 热路径分配

规则：

- 避免在 draw 里反复 sort 大 Vec。
- 避免在 draw 里执行 regex。
- 避免在 draw 里运行外部命令。
- 避免每帧 format 大量长字符串。
- 长文本截断、单位格式化可以在 collector 或 view-model 层预处理。

### 7.3 ViewModel 层

建议增加 `ui/view_model.rs`：

```rust
pub struct DashboardViewModel {
    pub header: HeaderVm,
    pub cards: Vec<MetricCardVm>,
    pub core_rows: Vec<CoreRowVm>,
    pub process_rows: Vec<ProcessRowVm>,
    pub status_line: String,
}
```

数据变更时构建 ViewModel；draw 时只渲染 ViewModel。这样可以减少每帧计算。

### 7.4 局部重绘说明

ratatui 通常是全屏 buffer diff 输出，逻辑上每次调用 draw 都会构建整屏 buffer。不要过早追求“真正局部重绘”，优先做到：

- 不在无变化时 draw。
- draw 足够快。
- 热路径不阻塞。

## 8. Rust 为什么能比 Go 更快

这部分不是口号，是工程事实：

- Rust 没有 GC，长时间 TUI 不会遇到 GC pause。
- Rust 可以精确控制分配，减少每帧 allocation。
- Rust enum / struct / pattern matching 非常适合构建低开销状态机。
- Rust async 没有 goroutine 栈增长和调度模型的额外不确定性，合理使用 Tokio 可以做到很稳定的延迟。
- Rust 可以用 `Arc<RwLock<T>>`、`watch`、`mpsc` 明确表达数据流，避免隐藏共享状态。
- Rust 编译期所有权检查能防止后台 task 和 UI 状态乱共享，长期维护更稳。

需要注意：Rust 不会自动让架构变快。把慢外部命令放在主 loop 里，Rust 也会卡；把采集和渲染解耦后，Rust 的优势才能体现出来。

## 9. 分阶段优化计划

### Phase 1：按键立即重绘

目标：解决最明显的“按键要等刷新”问题。

步骤：

1. 在 input 分支处理完事件后立即调用 `ui.draw(...)`。
2. `Refresh` 事件触发数据刷新时，不要阻塞普通页面切换。
3. 主题切换从重建 UI 改成 `ui.set_theme(...)`。
4. 加一个简单日志或 debug counter，记录 input event 到 draw 的耗时。

验收：页面切换、滚动、帮助弹窗按键后立即反馈。

### Phase 2：采集后台化

目标：主 loop 不再 await 慢采集。

步骤：

1. 新建 collector supervisor task。
2. 使用 `watch::channel<SystemData>` 发布最新快照。
3. 主 loop 收到 `data_rx.changed()` 后更新 history 并设置 dirty。
4. 删除主 loop 中的 `collect_all_data().await`。

验收：即使 powermetrics 慢，按键仍能切页和退出。

### Phase 3：slow command 独立化

目标：外部命令永远不进入 UI 热路径。

步骤：

1. `PowermetricsCollector` 独立 task。
2. `TerminalCollector` 独立 task。
3. `ioreg` / `pmset` / `lsof` 全部 timeout + cache。
4. 失败时保留旧数据，不阻塞 UI。

验收：拔掉 sudo 权限或让命令失败，UI 仍稳定流畅。

### Phase 4：dirty render loop

目标：避免无意义全屏重绘。

步骤：

1. 增加 `render_interval = 16ms/33ms`。
2. 增加 `dirty` 标记。
3. input/data/theme/resize 设置 dirty。
4. 无变化时跳过 draw。

验收：CPU 占用下降，按键依旧低延迟。

### Phase 5：ViewModel 和 draw 热路径瘦身

目标：复杂 dashboard 仍保持流畅。

步骤：

1. 进程排序移到 collector 或 view-model 更新时。
2. sparkline 数据预裁剪。
3. 单位格式化预处理。
4. draw 中不 clone 大 Vec。

验收：Overview、Processes、IO 页面 draw 时间稳定低于目标。

## 10. 建议代码形态

### 10.1 主 loop 伪代码

```rust
let mut render_tick = tokio::time::interval(Duration::from_millis(16));
let mut dirty = true;

loop {
    tokio::select! {
        Some(input) = input_rx.recv() => {
            handle_input(input, &mut app_state);
            dirty = true;
        }

        Ok(_) = data_rx.changed() => {
            app_state.data = data_rx.borrow().clone();
            app_state.history.update_from_system_data(&app_state.data);
            dirty = true;
        }

        _ = render_tick.tick() => {
            if dirty {
                ui.draw(&app_state.data, &app_state.history, &app_state.config)?;
                dirty = false;
            }
        }
    }
}
```

### 10.2 Collector task 伪代码

```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    loop {
        interval.tick().await;

        let snapshot = collector.collect_fast_snapshot().await;
        let snapshot = merge_with_power_cache(snapshot).await;
        let snapshot = merge_with_terminal_cache(snapshot).await;

        let _ = data_tx.send(snapshot);
    }
});
```

### 10.3 Slow task 伪代码

```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(2));

    loop {
        interval.tick().await;

        if let Ok(metrics) = timeout(Duration::from_millis(2000), collect_powermetrics()).await {
            power_cache.write().await.replace(metrics);
        }
    }
});
```

## 11. 观测与 Benchmark

必须增加简单观测，否则优化靠感觉。

建议指标：

- `input_events_total`
- `draw_count`
- `draw_duration_ms_avg`
- `draw_duration_ms_p95`
- `collector_duration_ms`
- `powermetrics_duration_ms`
- `terminal_lsof_duration_ms`
- `input_to_draw_ms`

Debug 模式下底部可显示：

```text
FPS 30 · draw 2.1ms p95 4.8ms · input→draw 6.4ms · collect 83ms · power 1412ms cached
```

## 12. 验收标准

- 快速按 `1..9` 页面切换无明显延迟。
- `q` 退出在 powermetrics/lsof 卡住时仍立即响应。
- 关闭 sudo 权限时 UI 不停顿、不刷屏。
- `lsof` 不会每秒执行。
- draw 没有明显闪屏。
- 普通页面 draw 平均 `< 4ms`。
- 输入到重绘 `< 16ms`。
- Rust 版本在同等信息密度下 CPU 占用、内存占用和响应稳定性优于 Go 版本。

## 13. 优先级

最先做：

1. 输入后立即重绘。
2. 采集后台化。
3. powermetrics 独立 task。
4. terminal/sysctl/lsof 独立 task。
5. dirty render loop。

这五项完成后，项目的响应体验会从“刷新驱动的监控器”变成“事件驱动的实时 cockpit”。Rust 的性能优势也会真正体现出来。
