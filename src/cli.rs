//! Command line. Every value is a typed argument, so `--help` lists it, a wrong value prints the
//! valid ones, and the generated shell completions offer them.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

const KEYS_HELP: &str = "\
Keys: a-z, 0-9, f1-f24, kp0-kp9, kp+ kp- kp* kp/ kpenter, enter esc backspace tab space minus
equal capslock print scrolllock pause insert home pageup delete end pagedown arrow-up arrow-down
arrow-left arrow-right. Modifiers: ctrl shift alt win, joined with +.";

/// Configure the Redragon M913 (Impact Elite): buttons, macros, DPI, lighting, polling rate.
///
/// Settings are stored in the mouse itself. A wireless M913 that hasn't moved for a while sleeps
/// and stops answering: move it and run the command again.
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show current settings; everything (the full config) when no part is given
    Get {
        #[command(subcommand)]
        what: Option<Get>,
    },
    /// Change one setting: polling rate, DPI, lighting, or a button (side_1 ... fire)
    Set {
        #[command(subcommand)]
        what: Set,
    },
    /// Write a whole config file (YAML or JSON); only the sections present are written
    Apply {
        /// Defaults to $XDG_CONFIG_HOME/m913ctl/config.yaml
        file: Option<PathBuf>,
    },
    /// Validate a config file without touching the mouse
    Check {
        /// Defaults to $XDG_CONFIG_HOME/m913ctl/config.yaml
        file: Option<PathBuf>,
    },
    /// Print the full config; same as `get` with nothing after it
    Dump,
    /// Save the mouse's settings memory to a file
    Backup { file: PathBuf },
    /// Write a backup back to the mouse
    Restore { file: PathBuf },
    /// Print shell completions, e.g. `m913ctl completions fish > ~/.config/fish/completions/m913ctl.fish`
    Completions { shell: Shell },
}

#[derive(Subcommand)]
pub enum Get {
    /// USB polling rate in Hz
    PollingRate,
    /// DPI levels and which one is active
    Dpi,
    /// Logo lighting
    Lighting,
    /// Every button's action
    Buttons,
    /// One button's action
    Button { button: Button },
}

#[derive(Subcommand)]
pub enum Set {
    /// USB polling rate in Hz
    PollingRate { rate: PollingRate },
    /// One DPI level
    Dpi {
        /// Which level, 1-5
        #[arg(value_parser = clap::value_parser!(u8).range(1..=5))]
        level: u8,
        /// 100-16000 in steps of 100
        #[arg(value_parser = parse_dpi)]
        dpi: u16,
    },
    /// All five DPI levels at once
    DpiLevels {
        /// Five values, 100-16000 in steps of 100
        #[arg(num_args = 5, value_parser = parse_dpi, required = true)]
        dpi: Vec<u16>,
    },
    /// Which DPI level is active
    ActiveDpi {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=5))]
        level: u8,
    },
    /// Logo lighting; give any of the options. Only visible with the mouse's eco mode off
    Lighting(Lighting),
    /// Thumb grid button 1
    #[command(name = "side_1", next_help_heading = "Buttons")]
    Side1 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 2
    #[command(name = "side_2", next_help_heading = "Buttons")]
    Side2 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 3
    #[command(name = "side_3", next_help_heading = "Buttons")]
    Side3 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 4
    #[command(name = "side_4", next_help_heading = "Buttons")]
    Side4 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 5
    #[command(name = "side_5", next_help_heading = "Buttons")]
    Side5 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 6
    #[command(name = "side_6", next_help_heading = "Buttons")]
    Side6 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 7
    #[command(name = "side_7", next_help_heading = "Buttons")]
    Side7 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 8
    #[command(name = "side_8", next_help_heading = "Buttons")]
    Side8 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 9
    #[command(name = "side_9", next_help_heading = "Buttons")]
    Side9 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 10
    #[command(name = "side_10", next_help_heading = "Buttons")]
    Side10 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 11
    #[command(name = "side_11", next_help_heading = "Buttons")]
    Side11 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 12
    #[command(name = "side_12", next_help_heading = "Buttons")]
    Side12 {
        #[command(subcommand)]
        action: Action,
    },
    /// The left button
    #[command(name = "left", next_help_heading = "Buttons")]
    Left {
        #[command(subcommand)]
        action: Action,
    },
    /// The right button
    #[command(name = "right", next_help_heading = "Buttons")]
    Right {
        #[command(subcommand)]
        action: Action,
    },
    /// The wheel click
    #[command(name = "middle", next_help_heading = "Buttons")]
    Middle {
        #[command(subcommand)]
        action: Action,
    },
    /// The small button beside the left click
    #[command(name = "fire", next_help_heading = "Buttons")]
    Fire {
        #[command(subcommand)]
        action: Action,
    },
}

impl Set {
    /// (config name, action) for the button variants.
    pub fn button(self) -> Result<(&'static str, Action), Self> {
        Ok(match self {
            Set::Side1 { action } => ("side_1", action),
            Set::Side2 { action } => ("side_2", action),
            Set::Side3 { action } => ("side_3", action),
            Set::Side4 { action } => ("side_4", action),
            Set::Side5 { action } => ("side_5", action),
            Set::Side6 { action } => ("side_6", action),
            Set::Side7 { action } => ("side_7", action),
            Set::Side8 { action } => ("side_8", action),
            Set::Side9 { action } => ("side_9", action),
            Set::Side10 { action } => ("side_10", action),
            Set::Side11 { action } => ("side_11", action),
            Set::Side12 { action } => ("side_12", action),
            Set::Left { action } => ("left", action),
            Set::Right { action } => ("right", action),
            Set::Middle { action } => ("middle", action),
            Set::Fire { action } => ("fire", action),
            other => return Err(other),
        })
    }
}

#[derive(Args)]
#[group(required = true, multiple = true)]
pub struct Lighting {
    #[arg(long)]
    pub mode: Option<LightMode>,
    /// #rrggbb, for steady and breathing
    #[arg(long, value_parser = parse_color)]
    pub color: Option<String>,
    /// 1-255, for steady
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub brightness: Option<u8>,
    /// 1-5, for breathing and neon
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=5))]
    pub speed: Option<u8>,
}

#[derive(Subcommand)]
pub enum Action {
    /// Left click
    Left,
    /// Right click
    Right,
    /// Middle click
    Middle,
    /// Back (mouse button 4)
    Back,
    /// Forward (mouse button 5)
    Forward,
    /// Do nothing
    Disable,
    /// DPI up
    #[command(name = "dpi+")]
    DpiUp,
    /// DPI down
    #[command(name = "dpi-")]
    DpiDown,
    /// Cycle through the DPI levels
    DpiLoop,
    /// Triple left click
    ThreeClick,
    /// Cycle the polling rate
    PollingRateSwitch,
    /// Lighting on/off
    RgbToggle,
    /// A key, optionally with modifiers, e.g. ctrl+shift+a
    #[command(after_help = KEYS_HELP)]
    Key { combo: String },
    /// A media key
    Media { key: MediaKey },
    /// Repeated left clicks
    RapidFire {
        /// Raw stored value, lower is faster
        speed: u8,
        times: u8,
    },
}

#[derive(ValueEnum, Clone, Copy)]
pub enum Button {
    #[value(name = "side_1")] Side1,
    #[value(name = "side_2")] Side2,
    #[value(name = "side_3")] Side3,
    #[value(name = "side_4")] Side4,
    #[value(name = "side_5")] Side5,
    #[value(name = "side_6")] Side6,
    #[value(name = "side_7")] Side7,
    #[value(name = "side_8")] Side8,
    #[value(name = "side_9")] Side9,
    #[value(name = "side_10")] Side10,
    #[value(name = "side_11")] Side11,
    #[value(name = "side_12")] Side12,
    Left,
    Right,
    Middle,
    /// The small button beside the left click
    Fire,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum PollingRate {
    #[value(name = "125")] Hz125,
    #[value(name = "250")] Hz250,
    #[value(name = "500")] Hz500,
    #[value(name = "1000")] Hz1000,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum LightMode {
    Off,
    Steady,
    Breathing,
    Neon,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum MediaKey {
    MediaPlayer, PlayPause, Next, Previous, Stop, Mute, VolumeUp, VolumeDown, Email, Calculator,
    MyComputer, WwwHome, WwwSearch, WwwForward, WwwBack, WwwStop, WwwRefresh, WwwFavorites,
}

/// The config-file name of a value, which is also its command-line name.
pub fn name<T: ValueEnum>(v: T) -> String {
    v.to_possible_value().unwrap().get_name().to_string()
}

fn parse_dpi(s: &str) -> Result<u16, String> {
    match s.parse::<u16>() {
        Ok(d) if (100..=16000).contains(&d) && d % 100 == 0 => Ok(d),
        _ => Err("must be 100-16000 in steps of 100".into()),
    }
}

fn parse_color(s: &str) -> Result<String, String> {
    let ok = s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit());
    if ok { Ok(s.to_lowercase()) } else { Err("must be #rrggbb".into()) }
}
