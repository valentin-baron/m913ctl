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

use clap::{CommandFactory, Parser};
use serde_yaml_ng::{Mapping, Value};

use cli::{name, Action, Cli, Command, Get, Set};
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
        Action::RapidFire { speed, times } => {
            map(&[("fire", map(&[("speed", speed.into()), ("times", times.into())]))])
        }
    }
}

fn set(mouse: &mut Mouse, what: Set) -> Result<(), String> {
    let conf = match what {
        Set::PollingRate { rate } => nested(&["polling_rate"], name(rate).parse::<u64>().unwrap().into()),
        Set::Dpi { level, dpi } => {
            let mut levels = dump::dpi_levels(mouse)?;
            levels[level as usize - 1] = dpi;
            nested(&["dpi", "levels"], levels.into_iter().map(Value::from).collect())
        }
        Set::DpiLevels { dpi } => nested(&["dpi", "levels"], dpi.into_iter().map(Value::from).collect()),
        Set::ActiveDpi { level } => nested(&["dpi", "active"], level.into()),
        Set::Lighting(l) => {
            let mut m = Mapping::new();
            if let Some(mode) = l.mode {
                m.insert("mode".into(), name(mode).into());
            }
            if let Some(c) = l.color {
                m.insert("color".into(), c.into());
            }
            if let Some(b) = l.brightness {
                m.insert("brightness".into(), b.into());
            }
            if let Some(s) = l.speed {
                m.insert("speed".into(), s.into());
            }
            nested(&["lighting"], Value::Mapping(m))
        }
        button => {
            let Ok((button, action)) = button.button() else { unreachable!() };
            nested(&["buttons", button], action_value(action))
        }
    };
    let text = serde_yaml_ng::to_string(&conf).map_err(|e| e.to_string())?;
    write(mouse, config::plan(&text)?)
}

fn get(mouse: &mut Mouse, what: Option<Get>) -> Result<(), String> {
    let full = dump::dump(mouse)?;
    let Some(what) = what else {
        print!("{full}");
        return Ok(());
    };
    // The dump is the one decoder; print the requested part as it appears there, so the output
    // can be pasted into a config file.
    let (section, key) = match what {
        Get::PollingRate => ("polling_rate", None),
        Get::Dpi => ("dpi", None),
        Get::Lighting => ("lighting", None),
        Get::Buttons => ("buttons", None),
        Get::Button { button } => ("buttons", Some(name(button))),
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
            clap_complete::generate(shell, &mut Cli::command(), "m913ctl", &mut std::io::stdout());
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("m913ctl: {e}");
            ExitCode::FAILURE
        }
    }
}
