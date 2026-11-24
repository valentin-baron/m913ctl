//! Command line. Every value is a typed argument, so `--help` lists it, a wrong value prints the
//! valid ones, and the generated shell completions offer them.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
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
    /// Change one setting: polling rate, DPI, lighting, or a button (1-12, left, right, middle, fire)
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
    /// DPI levels and which one is active, or one part of that
    Dpi { part: Option<DpiPart> },
    /// Logo lighting, or one part of it
    Lighting { part: Option<LightPart> },
    /// Every button's action
    Buttons,
    /// The macros stored in the mouse, which buttons can be set to (`set <BUTTON> macro <NAME>`)
    Macros,
    /// Thumb grid button 1
    #[command(name = "1")]
    Side1,
    /// Thumb grid button 2
    #[command(name = "2")]
    Side2,
    /// Thumb grid button 3
    #[command(name = "3")]
    Side3,
    /// Thumb grid button 4
    #[command(name = "4")]
    Side4,
    /// Thumb grid button 5
    #[command(name = "5")]
    Side5,
    /// Thumb grid button 6
    #[command(name = "6")]
    Side6,
    /// Thumb grid button 7
    #[command(name = "7")]
    Side7,
    /// Thumb grid button 8
    #[command(name = "8")]
    Side8,
    /// Thumb grid button 9
    #[command(name = "9")]
    Side9,
    /// Thumb grid button 10
    #[command(name = "10")]
    Side10,
    /// Thumb grid button 11
    #[command(name = "11")]
    Side11,
    /// Thumb grid button 12
    #[command(name = "12")]
    Side12,
    /// The left button
    #[command(name = "left")]
    Left,
    /// The right button
    #[command(name = "right")]
    Right,
    /// The wheel click
    #[command(name = "middle")]
    Middle,
    /// The small button beside the left click
    #[command(name = "fire")]
    Fire,
}

impl Get {
    /// The config name of a button variant.
    pub fn button(&self) -> Option<&'static str> {
        Some(match self {
            Get::Side1 => "1",
            Get::Side2 => "2",
            Get::Side3 => "3",
            Get::Side4 => "4",
            Get::Side5 => "5",
            Get::Side6 => "6",
            Get::Side7 => "7",
            Get::Side8 => "8",
            Get::Side9 => "9",
            Get::Side10 => "10",
            Get::Side11 => "11",
            Get::Side12 => "12",
            Get::Left => "left",
            Get::Right => "right",
            Get::Middle => "middle",
            Get::Fire => "fire",
            _ => return None,
        })
    }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum DpiPart {
    /// All five levels
    Levels,
    /// Which level is active
    Active,
    #[value(name = "1")] L1,
    #[value(name = "2")] L2,
    #[value(name = "3")] L3,
    #[value(name = "4")] L4,
    #[value(name = "5")] L5,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum LightPart {
    Mode,
    Color,
    Brightness,
    Speed,
}

#[derive(Subcommand)]
pub enum Set {
    /// USB polling rate in Hz
    PollingRate { rate: PollingRate },
    /// DPI levels and which one is active
    #[command(subcommand)]
    Dpi(SetDpi),
    /// Logo lighting. Only visible with the mouse's eco mode off
    #[command(subcommand)]
    Lighting(SetLighting),
    /// Thumb grid button 1
    #[command(name = "1")]
    Side1 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 2
    #[command(name = "2")]
    Side2 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 3
    #[command(name = "3")]
    Side3 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 4
    #[command(name = "4")]
    Side4 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 5
    #[command(name = "5")]
    Side5 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 6
    #[command(name = "6")]
    Side6 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 7
    #[command(name = "7")]
    Side7 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 8
    #[command(name = "8")]
    Side8 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 9
    #[command(name = "9")]
    Side9 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 10
    #[command(name = "10")]
    Side10 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 11
    #[command(name = "11")]
    Side11 {
        #[command(subcommand)]
        action: Action,
    },
    /// Thumb grid button 12
    #[command(name = "12")]
    Side12 {
        #[command(subcommand)]
        action: Action,
    },
    /// The left button
    #[command(name = "left")]
    Left {
        #[command(subcommand)]
        action: Action,
    },
    /// The right button
    #[command(name = "right")]
    Right {
        #[command(subcommand)]
        action: Action,
    },
    /// The wheel click
    #[command(name = "middle")]
    Middle {
        #[command(subcommand)]
        action: Action,
    },
    /// The small button beside the left click
    #[command(name = "fire")]
    Fire {
        #[command(subcommand)]
        action: Action,
    },
}

impl Set {
    /// (config name, action) for the button variants.
    pub fn button(self) -> Result<(&'static str, Action), Self> {
        Ok(match self {
            Set::Side1 { action } => ("1", action),
            Set::Side2 { action } => ("2", action),
            Set::Side3 { action } => ("3", action),
            Set::Side4 { action } => ("4", action),
            Set::Side5 { action } => ("5", action),
            Set::Side6 { action } => ("6", action),
            Set::Side7 { action } => ("7", action),
            Set::Side8 { action } => ("8", action),
            Set::Side9 { action } => ("9", action),
            Set::Side10 { action } => ("10", action),
            Set::Side11 { action } => ("11", action),
            Set::Side12 { action } => ("12", action),
            Set::Left { action } => ("left", action),
            Set::Right { action } => ("right", action),
            Set::Middle { action } => ("middle", action),
            Set::Fire { action } => ("fire", action),
            other => return Err(other),
        })
    }
}

#[derive(Subcommand)]
pub enum SetDpi {
    /// All five levels at once
    Levels {
        /// Five values, 100-16000 in steps of 100
        #[arg(num_args = 5, value_parser = parse_dpi, required = true)]
        dpi: Vec<u16>,
    },
    /// Which level is active
    Active {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=5))]
        level: u8,
    },
    /// Level 1
    #[command(name = "1")]
    L1 {
        /// 100-16000 in steps of 100
        #[arg(value_parser = parse_dpi)]
        dpi: u16,
    },
    /// Level 2
    #[command(name = "2")]
    L2 {
        /// 100-16000 in steps of 100
        #[arg(value_parser = parse_dpi)]
        dpi: u16,
    },
    /// Level 3
    #[command(name = "3")]
    L3 {
        /// 100-16000 in steps of 100
        #[arg(value_parser = parse_dpi)]
        dpi: u16,
    },
    /// Level 4
    #[command(name = "4")]
    L4 {
        /// 100-16000 in steps of 100
        #[arg(value_parser = parse_dpi)]
        dpi: u16,
    },
    /// Level 5
    #[command(name = "5")]
    L5 {
        /// 100-16000 in steps of 100
        #[arg(value_parser = parse_dpi)]
        dpi: u16,
    },
}

impl SetDpi {
    /// Level number and value for the single-level variants.
    pub fn single(&self) -> Option<(usize, u16)> {
        match *self {
            SetDpi::L1 { dpi } => Some((1, dpi)),
            SetDpi::L2 { dpi } => Some((2, dpi)),
            SetDpi::L3 { dpi } => Some((3, dpi)),
            SetDpi::L4 { dpi } => Some((4, dpi)),
            SetDpi::L5 { dpi } => Some((5, dpi)),
            _ => None,
        }
    }
}

#[derive(Subcommand)]
pub enum SetLighting {
    Mode { mode: LightMode },
    /// For steady and breathing
    Color {
        /// #rrggbb
        #[arg(value_parser = parse_color)]
        color: String,
    },
    /// For steady
    Brightness {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=255))]
        brightness: u8,
    },
    /// For breathing and neon
    Speed {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=5))]
        speed: u8,
    },
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
    /// A macro already stored in the mouse (see `get macros`; macros are written with `apply`)
    Macro {
        name: String,
        /// 1-253, while-held, or until-key (until any key is pressed)
        #[arg(long, default_value = "1", value_parser = parse_repeat)]
        repeat: u8,
    },
    /// Repeated left clicks
    RapidFire {
        /// Raw stored value, lower is faster
        speed: u8,
        times: u8,
    },
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

fn parse_repeat(s: &str) -> Result<u8, String> {
    match s {
        "while-held" => Ok(crate::protocol::REPEAT_WHILE_HELD),
        "until-key" => Ok(crate::protocol::REPEAT_UNTIL_KEY),
        _ => s.parse::<u8>().ok().filter(|n| (1..=253).contains(n))
            .ok_or_else(|| "must be 1-253, while-held or until-key".to_string()),
    }
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
