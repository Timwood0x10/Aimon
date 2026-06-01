use clap::{Arg, Command};
use libc::getuid;
use termion::{
    event::{Event, Key},
    input::TermRead,
};
use tokio::sync::mpsc as tokio_mpsc;

#[derive(Debug)]
pub struct CliArgs {
    pub refresh_rate: Option<u64>,
    /// Refresh interval in milliseconds (overrides --refresh if set)
    pub refresh_rate_ms: Option<u64>,
    pub minimal_mode: bool,
    pub config_file: Option<String>,
    pub theme: Option<String>,
    pub lang: Option<String>,
    pub json_output: bool,
    pub csv_output: bool,
    pub stream_format: Option<String>,
    /// Allow experimental fan control (requires SMC write access, feature gate)
    pub allow_fan_control: bool,
}

pub fn parse_args() -> CliArgs {
    let matches = Command::new("system-alert")
        .bin_name("sysalert")
        .about("Advanced macOS System Monitor with Apple Silicon optimizations")
        .author("Marky-Shi")
        .version("0.1.0")
        .arg(
            Arg::new("refresh")
                .short('r')
                .long("refresh")
                .value_name("SECONDS")
                .help("Set refresh rate in seconds (default: 1)")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("refresh-ms")
                .long("refresh-ms")
                .value_name("MILLIS")
                .help("Set refresh rate in milliseconds, overrides --refresh (min: 100, e.g. 500 = 0.5s)")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("minimal")
                .short('m')
                .long("minimal")
                .help("Use minimal display mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Specify a custom configuration file"),
        )
        .arg(
            Arg::new("theme")
                .short('t')
                .long("theme")
                .value_name("THEME")
                .help("Set the UI theme (cyberpunk, nord, dracula, tokyo_night, monokai, solarized_dark, gruvbox, catppuccin, one_dark)"),
        )
        .arg(
            Arg::new("lang")
                .long("lang")
                .value_name("LANG")
                .help("Set language (en, zh)"),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output metrics as JSON once and exit")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .help("Output metrics as CSV once and exit")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stream")
                .long("stream")
                .value_name("FORMAT")
                .help("Continuous output in given format (json, csv, prometheus)"),
        )
        .arg(
            Arg::new("allow-fan-control")
                .long("allow-fan-control")
                .help("Enable experimental fan control (requires SMC write access, compile with fan-control feature)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    CliArgs {
        refresh_rate: matches.get_one::<u64>("refresh").copied(),
        refresh_rate_ms: matches.get_one::<u64>("refresh-ms").copied(),
        minimal_mode: matches.get_flag("minimal"),
        config_file: matches.get_one::<String>("config").cloned(),
        theme: matches.get_one::<String>("theme").cloned(),
        lang: matches.get_one::<String>("lang").cloned(),
        json_output: matches.get_flag("json"),
        csv_output: matches.get_flag("csv"),
        stream_format: matches.get_one::<String>("stream").cloned(),
        allow_fan_control: matches.get_flag("allow-fan-control"),
    }
}

pub async fn check_root() -> Result<(), Box<dyn std::error::Error>> {
    if unsafe { getuid() != 0 } {
        return Err("This program requires root privileges to access system metrics.".into());
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Quit,
    NextTab,
    PreviousTab,
    ToggleNotifications,
    Refresh,
    ShowSessionReport,
    CycleTheme,
    /// Switch to next layout
    NextLayout,
    /// Switch to previous layout
    PreviousLayout,
    /// Scroll content up (vim k)
    ScrollUp,
    /// Scroll content down (vim j)
    ScrollDown,
    /// Jump to top of list (vim g)
    GoToTop,
    /// Jump to bottom of list (vim G)
    GoToBottom,
    /// Cycle sort order forward (vim s)
    CycleSortForward,
    /// Cycle sort order backward (vim S)
    CycleSortBackward,
    /// Toggle party mode
    TogglePartyMode,
    /// Toggle help overlay (?)
    ToggleHelp,
    /// Toggle time travel visualization
    ToggleTimeTravel,
    /// Toggle achievements display
    ToggleAchievements,
    /// Jump to a specific layout by number (0-9) or named shortcut
    JumpToLayout(u8),
    /// Search processes (/)
    SearchProcess,
    /// Kill selected process (F9)
    KillProcess,
}

pub async fn handle_input() -> tokio_mpsc::Receiver<InputEvent> {
    let (tx, rx) = tokio_mpsc::channel(32);

    // MUST use std::thread::spawn, NOT tokio::spawn!
    // stdin.events() is a blocking iterator that would freeze a tokio worker.
    // blocking_send() doesn't need async runtime scheduling.
    std::thread::spawn(move || {
        let stdin = std::io::stdin();

        for event in stdin.events().flatten() {
            let input_event = match event {
                // Quit
                Event::Key(Key::Char('q')) | Event::Key(Key::Ctrl('c')) => Some(InputEvent::Quit),

                // Layout navigation (vim h/l and arrow keys)
                Event::Key(Key::Char('h')) | Event::Key(Key::Left) => {
                    Some(InputEvent::PreviousLayout)
                }
                Event::Key(Key::Char('l')) | Event::Key(Key::Right) => Some(InputEvent::NextLayout),

                // Scroll navigation (vim j/k and arrow keys)
                Event::Key(Key::Char('j')) | Event::Key(Key::Down) => Some(InputEvent::ScrollDown),
                Event::Key(Key::Char('k')) | Event::Key(Key::Up) => Some(InputEvent::ScrollUp),

                // Jump to top/bottom (vim g/G)
                Event::Key(Key::Char('g')) => Some(InputEvent::GoToTop),
                Event::Key(Key::Char('G')) => Some(InputEvent::GoToBottom),

                // Quick layout jump (0-9)
                Event::Key(Key::Char('0')) => Some(InputEvent::JumpToLayout(0)),
                Event::Key(Key::Char('1')) => Some(InputEvent::JumpToLayout(1)),
                Event::Key(Key::Char('2')) => Some(InputEvent::JumpToLayout(2)),
                Event::Key(Key::Char('3')) => Some(InputEvent::JumpToLayout(3)),
                Event::Key(Key::Char('4')) => Some(InputEvent::JumpToLayout(4)),
                Event::Key(Key::Char('5')) => Some(InputEvent::JumpToLayout(5)),
                Event::Key(Key::Char('6')) => Some(InputEvent::JumpToLayout(6)),
                Event::Key(Key::Char('7')) => Some(InputEvent::JumpToLayout(7)),
                Event::Key(Key::Char('8')) => Some(InputEvent::JumpToLayout(8)),
                Event::Key(Key::Char('9')) => Some(InputEvent::JumpToLayout(9)),
                Event::Key(Key::Char('d')) => Some(InputEvent::JumpToLayout(10)),
                Event::Key(Key::Char('\n')) | Event::Key(Key::Char(' ')) => {
                    Some(InputEvent::JumpToLayout(1))
                }

                // Sort cycling (vim s/S)
                Event::Key(Key::Char('s')) => Some(InputEvent::CycleSortForward),
                Event::Key(Key::Char('S')) => Some(InputEvent::CycleSortBackward),

                // Search and kill
                Event::Key(Key::Char('/')) => Some(InputEvent::SearchProcess),
                Event::Key(Key::F(9)) => Some(InputEvent::KillProcess),

                // Help overlay
                Event::Key(Key::Char('?')) => Some(InputEvent::ToggleHelp),

                // Existing bindings
                Event::Key(Key::Char('n')) => Some(InputEvent::ToggleNotifications),
                Event::Key(Key::Char('r')) => Some(InputEvent::Refresh),
                Event::Key(Key::Char('R')) => Some(InputEvent::ShowSessionReport),
                Event::Key(Key::Char('t')) => Some(InputEvent::CycleTheme),
                Event::Key(Key::Char('\t')) => Some(InputEvent::NextLayout),
                Event::Key(Key::BackTab) => Some(InputEvent::PreviousLayout),

                _ => None,
            };

            if let Some(event) = input_event {
                if tx.blocking_send(event).is_err() {
                    break;
                }
            }
        }
    });

    rx
}
