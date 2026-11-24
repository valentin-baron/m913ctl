//! m913ctl: configure the Redragon M913 (Impact Elite) on Linux, Windows and macOS.
//!
//! The protocol was captured with usbmon from Redragon's Windows tool: every request is feature
//! report 0x08, answered by input report 0x09 echoing it.
//!   08 08 00 HI LO N            read N (<=10) bytes   -> 09 08 00 HI LO N <data>
//!   08 07 00 HI LO N <data>     write N (<=10) bytes  -> echoed
//! Byte 15 is a checksum. The vendor tool's Apply is only writes; there is no commit command.

mod cli;
mod config;
mod device;
mod dump;
mod protocol;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, FromArgMatches};
use serde_yaml_ng::{Mapping, Value};

use cli::{name, Action, Cli, Command, DpiPart, Get, LightPart, Set, SetDpi, SetLighting};
use device::Mouse;

fn default_config() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"));
    base.join("m913ctl/config.yaml")
}

fn plan_file(path: &Path) -> Result<Vec<config::Write>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    config::plan(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn write(mouse: &mut Mouse, writes: Vec<config::Write>) -> Result<(), String> {
    for w in writes {
        mouse.write(w.addr, &w.data)?;
        if !w.what.is_empty() {
            println!("{}", w.what);
        }
    }
    Ok(())
}

/// `{a: {b: v}}` from a path, for handing one setting to the config planner.
fn nested(path: &[&str], v: Value) -> Value {
    path.iter().rev().fold(v, |inner, key| {
        let mut m = Mapping::new();
        m.insert(Value::from(*key), inner);
        Value::Mapping(m)
    })
}

fn map(pairs: &[(&str, Value)]) -> Value {
    Value::Mapping(pairs.iter().map(|(k, v)| (Value::from(*k), v.clone())).collect())
}

fn action_value(action: Action) -> Value {
    let simple = |s: &str| Value::from(s);
    match action {
        Action::Left => simple("left"),
        Action::Right => simple("right"),
        Action::Middle => simple("middle"),
        Action::Back => simple("back"),
        Action::Forward => simple("forward"),
        Action::Disable => simple("disable"),
        Action::DpiUp => simple("dpi+"),
        Action::DpiDown => simple("dpi-"),
        Action::DpiLoop => simple("dpi-loop"),
        Action::ThreeClick => simple("three-click"),
        Action::PollingRateSwitch => simple("polling-rate-switch"),
        Action::RgbToggle => simple("rgb-toggle"),
        Action::Key { combo } => map(&[("key", combo.into())]),
        Action::Media { key } => map(&[("media", name(key).into())]),
        Action::Macro { .. } => unreachable!("macros are set by index in set()"),
        Action::RapidFire { speed, times } => {
            map(&[("fire", map(&[("speed", speed.into()), ("times", times.into())]))])
        }
    }
}

fn set(mouse: &mut Mouse, what: Set) -> Result<(), String> {
    let conf = match what {
        Set::PollingRate { rate } => nested(&["polling_rate"], name(rate).parse::<u64>().unwrap().into()),
        Set::Dpi(SetDpi::Levels { dpi }) => nested(&["dpi", "levels"], dpi.into_iter().map(Value::from).collect()),
        Set::Dpi(SetDpi::Active { level }) => nested(&["dpi", "active"], level.into()),
        Set::Dpi(one) => {
            let (level, dpi) = one.single().expect("the other SetDpi variants are matched above");
            let mut levels = dump::dpi_levels(mouse)?;
            levels[level - 1] = dpi;
            nested(&["dpi", "levels"], levels.into_iter().map(Value::from).collect())
        }
        Set::Lighting(l) => {
            let (key, value): (&str, Value) = match l {
                SetLighting::Mode { mode } => ("mode", name(mode).into()),
                SetLighting::Color { color } => ("color", color.into()),
                SetLighting::Brightness { brightness } => ("brightness", brightness.into()),
                SetLighting::Speed { speed } => ("speed", speed.into()),
            };
            nested(&["lighting", key], value)
        }
        button => {
            let Ok((button, action)) = button.button() else { unreachable!("all other Set variants are matched above") };
            if let Action::Macro { name, repeat } = action {
                return set_macro(mouse, button, name.as_deref(), repeat);
            }
            nested(&["buttons", button], action_value(action))
        }
    };
    let text = serde_yaml_ng::to_string(&conf).map_err(|e| e.to_string())?;
    write(mouse, config::plan(&text)?)
}

/// Point a button at a macro already in the mouse; the config planner only knows the macros of
/// its own file.
fn set_macro(mouse: &mut Mouse, button: &str, name: Option<&str>, repeat: u8) -> Result<(), String> {
    let stored = dump::stored_macros(mouse)?;
    let Some(name) = name else {
        // Like clap's own help for a missing argument, but the possible values come from the mouse.
        let mut cmd = command();
        cmd.build();
        let sub = ["set", button, "macro"].iter().fold(&mut cmd, |c, n| c.find_subcommand_mut(n).expect("known path"));
        print!("{}", sub.render_help());
        println!("\nMacros in the mouse:");
        if stored.is_empty() {
            println!("  none yet; define them under `macros:` in a config and `apply` it");
        }
        for (i, n) in &stored {
            let (_, steps) = dump::read_macro(mouse, *i)?;
            println!("  {n}: [{}]", steps.join(", "));
        }
        std::process::exit(2);
    };
    let Some((index, _)) = stored.iter().find(|(_, n)| n == name) else {
        let names: Vec<_> = stored.iter().map(|(_, n)| n.as_str()).collect();
        return Err(if names.is_empty() {
            "the mouse holds no macros; define them under `macros:` in a config and `apply` it".into()
        } else {
            format!("no macro {name:?} in the mouse (it has: {})", names.join(", "))
        });
    };
    let rec = protocol::lookup(&protocol::BUTTONS, button).expect("button names come from the CLI");
    let record = protocol::closed(&[protocol::T_MACRO, *index as u8, repeat]);
    mouse.write(protocol::ADDR_RECORDS + 4 * rec as u16, &record)?;
    let repeat = match repeat {
        protocol::REPEAT_WHILE_HELD => "while-held".to_string(),
        protocol::REPEAT_UNTIL_KEY => "until-key".to_string(),
        n => n.to_string(),
    };
    println!("{button}: macro {name}, repeat {repeat}");
    Ok(())
}

fn get(mouse: &mut Mouse, what: Option<Get>) -> Result<(), String> {
    // Single values come straight from memory: the dump leaves out what the current mode
    // doesn't use (the colour while the light is off, say).
    match what {
        Some(Get::Dpi { part: Some(part) }) => {
            let levels = dump::dpi_levels(mouse)?;
            let list = || levels.iter().map(u16::to_string).collect::<Vec<_>>().join(", ");
            match part {
                DpiPart::Levels => println!("[{}]", list()),
                DpiPart::Active => println!("{}", mouse.read(protocol::ADDR_DPI_ACTIVE, 1)?[0]),
                level => println!("{}", levels[name(level).parse::<usize>().unwrap() - 1]),
            }
            return Ok(());
        }
        Some(Get::Macros) => {
            for (i, _) in dump::stored_macros(mouse)? {
                let (name, steps) = dump::read_macro(mouse, i)?;
                println!("{name}: [{}]", steps.join(", "));
            }
            return Ok(());
        }
        Some(Get::Lighting { part: Some(part) }) => {
            let mem = mouse.read(protocol::ADDR_COLOR, 9)?;
            let at = |addr: u16| mem[(addr - protocol::ADDR_COLOR) as usize];
            match part {
                LightPart::Mode => {
                    let m = at(protocol::ADDR_LIGHT_MODE) as usize;
                    println!("{}", protocol::LIGHT_MODES.get(m).copied().unwrap_or("unknown"));
                }
                LightPart::Color => println!("#{:02x}{:02x}{:02x}", mem[0], mem[1], mem[2]),
                LightPart::Brightness => println!("{}", at(protocol::ADDR_BRIGHTNESS)),
                LightPart::Speed => println!("{}", at(protocol::ADDR_SPEED)),
            }
            return Ok(());
        }
        _ => {}
    }

    let full = dump::dump(mouse)?;
    let Some(what) = what else {
        print!("{full}");
        return Ok(());
    };
    // Sections and buttons are printed as they appear in the dump, so the output can be pasted
    // into a config file.
    let (section, key) = match what {
        Get::PollingRate => ("polling_rate", None),
        Get::Dpi { .. } => ("dpi", None),
        Get::Lighting { .. } => ("lighting", None),
        Get::Buttons => ("buttons", None),
        Get::Macros => unreachable!("handled above"),
        button => ("buttons", button.button()),
    };
    let lines: Vec<&str> = full.lines().collect();
    let at = lines.iter().position(|l| l.starts_with(&format!("{section}:"))).ok_or("not in the dump")?;
    let inline = lines[at][section.len() + 1..].trim();
    let body: Vec<&str> = lines[at + 1..].iter().take_while(|l| l.starts_with("  ")).map(|l| &l[2..]).collect();
    match key {
        Some(key) => {
            let line = body.iter().find(|l| l.starts_with(&format!("{key}:"))).ok_or("not in the dump")?;
            println!("{}", line[key.len() + 1..].trim());
        }
        None if !inline.is_empty() => println!("{inline}"),
        None => println!("{}", body.join("\n")),
    }
    Ok(())
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Get { what } => get(&mut Mouse::open()?, what)?,
        Command::Dump => get(&mut Mouse::open()?, None)?,
        Command::Set { what } => set(&mut Mouse::open()?, what)?,
        Command::Check { file } => {
            let path = file.unwrap_or_else(default_config);
            let writes = plan_file(&path)?;
            println!("{}: ok ({} writes)", path.display(), writes.len());
        }
        Command::Apply { file } => {
            let writes = plan_file(&file.unwrap_or_else(default_config))?;
            write(&mut Mouse::open()?, writes)?;
        }
        Command::Backup { file } => {
            let mem = Mouse::open()?.read(0, protocol::BACKUP_END)?;
            std::fs::write(&file, mem).map_err(|e| format!("{}: {e}", file.display()))?;
        }
        Command::Restore { file } => {
            let mem = std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            if mem.len() != protocol::BACKUP_END {
                return Err(format!("{} is not an m913ctl backup", file.display()));
            }
            let mut mouse = Mouse::open()?;
            for (addr, len) in protocol::BACKUP_RANGES {
                mouse.write(addr, &mem[addr as usize..addr as usize + len])?;
            }
        }
        Command::Completions { shell } => {
            clap_complete::generate(shell, &mut command(), "m913ctl", &mut std::io::stdout());
        }
    }
    Ok(())
}

/// The CLI, with every command that needs an argument printing its full help (possible values,
/// key names) when called without one, instead of a bare "required argument missing".
fn command() -> clap::Command {
    fn help_when_empty(c: clap::Command) -> clap::Command {
        let needs = c.is_subcommand_required_set()
            || c.get_arguments().any(|a| a.is_required_set() && a.is_positional());
        c.arg_required_else_help(needs).mut_subcommands(help_when_empty)
    }
    help_when_empty(Cli::command())
}

fn main() -> ExitCode {
    let cli = Cli::from_arg_matches(&command().get_matches()).unwrap_or_else(|e| e.exit());
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("m913ctl: {e}");
            ExitCode::FAILURE
        }
    }
}
